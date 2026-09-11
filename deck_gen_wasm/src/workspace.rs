//! Reactive workspace: tree, selection, and the actions the UI triggers.
//!
//! [`Workspace`] is a cheap `Copy` handle to Leptos signals (VFS, selection,
//! expanded folders, status, loading). Explorer, editor, and the activity bar
//! all clone it. Mutations go through the VFS; async work (`prepare_html`,
//! `prepare_pdf`, folder pick, template fetch, ZIP) uses `spawn_local` and the
//! `loading` flag so two long actions cannot overlap.

use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::export::{save_zip_bytes, vfs_to_zip, ZIP_FILENAME};
use crate::fs::{join_path, parent_path, rewrite_prefix, Vfs, VfsFs};
use crate::load_folder::{install_folder, pick_and_read_folder, PickResult};
use crate::persist::{save_session, Session};
use crate::template::install_new_game;

/// Whether an editor tab shows the source or a rendered preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabKind {
    /// Editable source.
    Edit,
    /// Rendered HTML / Markdown preview.
    Preview,
}

/// One open editor tab (path + edit vs preview).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTab {
    /// Workspace path of the file.
    pub path: String,
    /// Source vs preview.
    pub kind: TabKind,
}

/// Shared editor state. Cheap to copy: every field is a signal.
#[derive(Clone, Copy)]
pub struct Workspace {
    /// In-memory file tree.
    pub vfs: RwSignal<Vfs>,
    /// Explorer selection (file or folder path).
    pub selected: RwSignal<Option<String>>,
    /// Open editor tabs, left to right.
    pub tabs: RwSignal<Vec<OpenTab>>,
    /// Focused tab, if any.
    pub active_tab: RwSignal<Option<OpenTab>>,
    /// Directories currently expanded in the tree.
    pub expanded: RwSignal<HashSet<String>>,
    /// Footer status line.
    pub status: RwSignal<String>,
    /// True while an async action (load / template / HTML / PDF / ZIP) is running.
    pub loading: RwSignal<bool>,
}

impl Workspace {
    /// Restore signals from a localStorage snapshot (or an empty default).
    pub fn from_session(session: Session) -> Self {
        let expanded = session.expanded_set();
        let (tabs, active_tab) = initial_tabs(&session.vfs, &session.selected);
        Self {
            vfs: RwSignal::new(session.vfs),
            selected: RwSignal::new(session.selected),
            tabs: RwSignal::new(tabs),
            active_tab: RwSignal::new(active_tab),
            expanded: RwSignal::new(expanded),
            status: RwSignal::new(String::new()),
            loading: RwSignal::new(false),
        }
    }

    /// Serializable snapshot for localStorage.
    pub fn snapshot(&self) -> Session {
        Session::from_workspace(self.vfs.get(), self.selected.get(), &self.expanded.get())
    }

    /// Select `path`; folders also toggle expansion. Files open an edit tab.
    pub fn select(&self, path: String, is_dir: bool) {
        self.selected.set(Some(path.clone()));
        self.status.set(String::new());
        if is_dir {
            self.expanded.update(|set| {
                if !set.remove(&path) {
                    set.insert(path);
                }
            });
        } else {
            self.open_tab(OpenTab {
                path,
                kind: TabKind::Edit,
            });
        }
    }

    /// Focus `tab`, creating it if it is not already open.
    pub fn open_tab(&self, tab: OpenTab) {
        self.tabs.update({
            let tab = tab.clone();
            move |tabs| {
                if !tabs.contains(&tab) {
                    tabs.push(tab);
                }
            }
        });
        self.active_tab.set(Some(tab));
    }

    /// Make an existing tab active and select its path in the explorer.
    pub fn activate_tab(&self, tab: OpenTab) {
        self.active_tab.set(Some(tab.clone()));
        self.selected.set(Some(tab.path));
    }

    /// Close `tab`. If it was active, activate the neighbor to the right (else left).
    pub fn close_tab(&self, tab: OpenTab) {
        let mut tabs = self.tabs.get();
        let idx = tabs.iter().position(|open| open == &tab);
        tabs.retain(|open| open != &tab);
        let was_active = self.active_tab.get().as_ref() == Some(&tab);
        self.tabs.set(tabs.clone());
        if !was_active {
            return;
        }
        let next = idx.and_then(|i| {
            if i < tabs.len() {
                tabs.get(i).cloned()
            } else {
                tabs.last().cloned()
            }
        });
        self.active_tab.set(next.clone());
        if let Some(next) = next {
            self.selected.set(Some(next.path));
        }
    }

    /// Prompt for a name and create an empty file under the creation parent.
    pub fn create_file(&self) {
        let Some(name) = ask_name("New file name") else {
            return;
        };
        let parent = self.creation_parent();
        let path = join_path(&parent, &name);
        match self.vfs.try_update(|vfs| vfs.create_file(&path)) {
            Some(Ok(())) => {
                self.expand_ancestors(&parent);
                self.selected.set(Some(path.clone()));
                self.open_tab(OpenTab {
                    path,
                    kind: TabKind::Edit,
                });
                self.status.set(String::new());
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set("Could not update workspace".into()),
        }
    }

    /// Prompt for a name and create a folder under the creation parent.
    pub fn create_folder(&self) {
        let Some(name) = ask_name("New folder name") else {
            return;
        };
        let parent = self.creation_parent();
        let path = join_path(&parent, &name);
        match self.vfs.try_update(|vfs| vfs.mkdir(&path)) {
            Some(Ok(())) => {
                self.expand_ancestors(&path);
                self.selected.set(Some(path));
                self.status.set(String::new());
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set("Could not update workspace".into()),
        }
    }

    /// Write the current snapshot to localStorage (manual Save).
    pub fn persist(&self) {
        match save_session(&self.snapshot()) {
            Ok(()) => self.status.set("Saved in this browser".into()),
            Err(err) => self.status.set(err),
        }
    }

    /// Empty the tree and persist that empty session.
    pub fn clear(&self) {
        self.vfs.set(Vfs::default());
        self.selected.set(None);
        self.tabs.set(Vec::new());
        self.active_tab.set(None);
        self.expanded.set(HashSet::new());
        self.status.set("Workspace cleared".into());
        let _ = save_session(&self.snapshot());
    }

    /// Pick a local folder and copy it under `games/` with a unique name.
    pub fn load_game_from_disk(&self, warning: RwSignal<Option<String>>) {
        if self.loading.get() {
            return;
        }
        self.loading.set(true);
        self.status.set("Select a folder…".into());
        let workspace = *self;
        spawn_local(async move {
            match pick_and_read_folder().await {
                PickResult::Cancelled => {
                    workspace.status.set(String::new());
                }
                PickResult::Rejected(reason) => {
                    warning.set(Some(reason));
                    workspace.status.set(String::new());
                }
                PickResult::Ready { name, files, dirs } => {
                    workspace.status.set("Loading folder…".into());
                    let mut vfs = workspace.vfs.get_untracked();
                    match install_folder(&mut vfs, &name, &dirs, &files) {
                        Ok(folder) => {
                            workspace.vfs.set(vfs);
                            let path = format!("games/{folder}");
                            workspace.expand_ancestors(&path);
                            workspace.selected.set(Some(path.clone()));
                            workspace.status.set(format!("Loaded {path}"));
                        }
                        Err(err) => workspace.status.set(err),
                    }
                }
            }
            workspace.loading.set(false);
        });
    }

    /// Run [`deck_gen::prepare_html`] on the VFS after a 0ms yield so the UI can paint.
    pub fn prepare_html(&self, warning: RwSignal<Option<String>>) {
        if self.loading.get() {
            return;
        }
        self.loading.set(true);
        self.status.set("Preparing HTML…".into());
        let workspace = *self;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            let fs = Arc::new(VfsFs::new(workspace.vfs.get_untracked()));
            match deck_gen::prepare_html(fs.clone()) {
                Ok(n) => {
                    workspace.vfs.set(take_vfs(fs));
                    workspace.status.set(format!("Prepared HTML for {n} decks"));
                }
                Err(err) => {
                    warning.set(Some(err.to_string()));
                    workspace.status.set(String::new());
                }
            }
            workspace.loading.set(false);
        });
    }

    /// Run [`deck_gen::prepare_pdf`] with the browser engine after a 0ms yield.
    pub fn prepare_pdf(&self, warning: RwSignal<Option<String>>) {
        if self.loading.get() {
            return;
        }
        self.loading.set(true);
        self.status.set("Preparing PDF…".into());
        let workspace = *self;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            let fs = Arc::new(VfsFs::new(workspace.vfs.get_untracked()));
            let engine = prepare_pdf_web::WebPdfEngine;
            match deck_gen::prepare_pdf(fs.clone(), &engine).await {
                Ok(n) => {
                    workspace.vfs.set(take_vfs(fs));
                    workspace.status.set(format!("Prepared PDF for {n} decks"));
                }
                Err(err) => {
                    warning.set(Some(err.to_string()));
                    workspace.status.set(String::new());
                }
            }
            workspace.loading.set(false);
        });
    }

    /// Fetch the `new-game` template and install it under `games/`.
    pub fn add_new_game(&self) {
        if self.loading.get() {
            return;
        }
        self.loading.set(true);
        self.status.set("Loading new-game template…".into());
        let workspace = *self;
        spawn_local(async move {
            let mut vfs = workspace.vfs.get_untracked();
            let result = install_new_game(&mut vfs).await;
            match result {
                Ok(installed) => {
                    workspace.vfs.set(vfs);
                    let path = format!("games/{}", installed.folder);
                    workspace.expand_ancestors(&path);
                    workspace.selected.set(Some(path.clone()));
                    workspace.status.set(format!(
                        "Added {path} from {}",
                        installed.source
                    ));
                }
                Err(err) => workspace.status.set(err),
            }
            workspace.loading.set(false);
        });
    }

    /// Encode the tree as ZIP and offer it to the browser.
    pub fn download(&self) {
        let vfs = self.vfs.get();
        match vfs_to_zip(&vfs) {
            Ok(bytes) => {
                self.status.set("Downloading ZIP…".into());
                let status = self.status;
                spawn_local(async move {
                    match save_zip_bytes(bytes, ZIP_FILENAME).await {
                        Ok(()) => status.set(format!("Downloaded {ZIP_FILENAME}")),
                        Err(err) => status.set(err),
                    }
                });
            }
            Err(err) => self.status.set(err),
        }
    }

    /// Context-menu command (`rename` / `delete` / `preview`) on an explorer entry.
    pub fn run_entry_command(&self, id: &str, path: &str) {
        match id {
            "delete" => self.delete_entry(path),
            "rename" => self.rename_entry(path),
            "preview" => self.open_tab(OpenTab {
                path: path.to_string(),
                kind: TabKind::Preview,
            }),
            other => self.status.set(format!("Unknown command {other}")),
        }
    }

    fn delete_entry(&self, path: &str) {
        match self.vfs.try_update(|vfs| vfs.remove(path)) {
            Some(Ok(())) => {
                self.forget_path(path);
                self.status.set(format!("Deleted {path}"));
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set("Could not update workspace".into()),
        }
    }

    fn rename_entry(&self, path: &str) {
        let Some(name) = ask_name("New name") else {
            return;
        };
        match self.vfs.try_update(|vfs| vfs.rename(path, &name)) {
            Some(Ok(new_path)) => {
                self.rewrite_paths(path, &new_path);
                self.status.set(format!("Renamed to {new_path}"));
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set("Could not update workspace".into()),
        }
    }

    fn forget_path(&self, path: &str) {
        let prefix = format!("{path}/");
        let gone = |current: &str| current == path || current.starts_with(&prefix);
        self.selected.update(|selected| {
            if selected.as_ref().is_some_and(|current| gone(current)) {
                *selected = None;
            }
        });
        self.expanded.update(|set| {
            set.retain(|current| !gone(current));
        });
        let tabs = self.tabs.get();
        let active = self.active_tab.get();
        let closing_active = active.as_ref().is_some_and(|tab| gone(&tab.path));
        let idx = active.as_ref().and_then(|tab| tabs.iter().position(|open| open == tab));
        let remaining: Vec<_> = tabs.into_iter().filter(|tab| !gone(&tab.path)).collect();
        let next = if closing_active {
            idx.and_then(|i| {
                if remaining.is_empty() {
                    None
                } else {
                    remaining.get(i.min(remaining.len() - 1)).cloned()
                }
            })
        } else {
            active.filter(|tab| remaining.contains(tab))
        };
        self.tabs.set(remaining);
        self.active_tab.set(next.clone());
        if let Some(next) = next {
            self.selected.set(Some(next.path));
        }
    }

    fn rewrite_paths(&self, old: &str, new: &str) {
        self.selected.update(|selected| {
            if let Some(current) = selected.as_ref() {
                *selected = Some(rewrite_prefix(current, old, new));
            }
        });
        self.expanded.update(|set| {
            *set = set
                .iter()
                .map(|current| rewrite_prefix(current, old, new))
                .collect();
        });
        self.tabs.update(|tabs| {
            for tab in tabs.iter_mut() {
                tab.path = rewrite_prefix(&tab.path, old, new);
            }
        });
        self.active_tab.update(|active| {
            if let Some(tab) = active {
                tab.path = rewrite_prefix(&tab.path, old, new);
            }
        });
    }

    fn creation_parent(&self) -> String {
        match self.selected.get() {
            Some(path) if self.vfs.get().is_dir(&path) => path,
            Some(path) => parent_path(&path),
            None => String::new(),
        }
    }

    fn expand_ancestors(&self, path: &str) {
        self.expanded.update(|set| {
            let mut current = path.to_string();
            while !current.is_empty() {
                set.insert(current.clone());
                current = parent_path(&current);
            }
        });
    }
}

fn take_vfs(fs: Arc<VfsFs>) -> crate::fs::Vfs {
    match Arc::try_unwrap(fs) {
        Ok(inner) => inner.into_vfs(),
        Err(arc) => arc.clone_vfs(),
    }
}

fn initial_tabs(vfs: &Vfs, selected: &Option<String>) -> (Vec<OpenTab>, Option<OpenTab>) {
    match selected {
        Some(path) if vfs.is_file(path) => {
            let tab = OpenTab {
                path: path.clone(),
                kind: TabKind::Edit,
            };
            (vec![tab.clone()], Some(tab))
        }
        _ => (Vec::new(), None),
    }
}

fn ask_name(message: &str) -> Option<String> {
    let window = web_sys::window()?;
    window
        .prompt_with_message(message)
        .ok()
        .flatten()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}
