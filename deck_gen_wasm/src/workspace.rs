//! Reactive workspace: tree, selection, and the actions the UI triggers.

use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::export::{save_zip_bytes, vfs_to_zip, ZIP_FILENAME};
use crate::fs::{join_path, parent_path, rewrite_prefix, Vfs, VfsFs};
use crate::load_folder::{install_folder, pick_and_read_folder, PickResult};
use crate::persist::{save_session, Session};
use crate::template::install_new_game;

#[derive(Clone, Copy)]
pub struct Workspace {
    pub vfs: RwSignal<Vfs>,
    pub selected: RwSignal<Option<String>>,
    pub expanded: RwSignal<HashSet<String>>,
    pub status: RwSignal<String>,
    pub loading: RwSignal<bool>,
}

impl Workspace {
    pub fn from_session(session: Session) -> Self {
        let expanded = session.expanded_set();
        Self {
            vfs: RwSignal::new(session.vfs),
            selected: RwSignal::new(session.selected),
            expanded: RwSignal::new(expanded),
            status: RwSignal::new(String::new()),
            loading: RwSignal::new(false),
        }
    }

    pub fn snapshot(&self) -> Session {
        Session::from_workspace(self.vfs.get(), self.selected.get(), &self.expanded.get())
    }

    pub fn open_file_path(&self) -> Option<String> {
        self.selected
            .get()
            .filter(|path| self.vfs.get().is_file(path))
    }

    pub fn select(&self, path: String, is_dir: bool) {
        self.selected.set(Some(path.clone()));
        self.status.set(String::new());
        if is_dir {
            self.expanded.update(|set| {
                if !set.remove(&path) {
                    set.insert(path);
                }
            });
        }
    }

    pub fn create_file(&self) {
        let Some(name) = ask_name("New file name") else {
            return;
        };
        let parent = self.creation_parent();
        let path = join_path(&parent, &name);
        match self.vfs.try_update(|vfs| vfs.create_file(&path)) {
            Some(Ok(())) => {
                self.expand_ancestors(&parent);
                self.selected.set(Some(path));
                self.status.set(String::new());
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set("Could not update workspace".into()),
        }
    }

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

    pub fn persist(&self) {
        match save_session(&self.snapshot()) {
            Ok(()) => self.status.set("Saved in this browser".into()),
            Err(err) => self.status.set(err),
        }
    }

    pub fn clear(&self) {
        self.vfs.set(Vfs::default());
        self.selected.set(None);
        self.expanded.set(HashSet::new());
        self.status.set("Workspace cleared".into());
        let _ = save_session(&self.snapshot());
    }

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

    pub fn run_entry_command(&self, id: &str, path: &str) {
        match id {
            "delete" => self.delete_entry(path),
            "rename" => self.rename_entry(path),
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
        self.selected.update(|selected| {
            if selected
                .as_ref()
                .is_some_and(|current| current == path || current.starts_with(&format!("{path}/")))
            {
                *selected = None;
            }
        });
        self.expanded.update(|set| {
            set.retain(|current| current != path && !current.starts_with(&format!("{path}/")));
        });
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

fn ask_name(message: &str) -> Option<String> {
    let window = web_sys::window()?;
    window
        .prompt_with_message(message)
        .ok()
        .flatten()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}
