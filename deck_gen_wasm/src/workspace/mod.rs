//! Reactive workspace: tree, selection, and the actions the UI triggers.
//!
//! [`Workspace`] is a cheap `Copy` handle to Leptos signals (VFS, selection,
//! expanded folders, status, loading). Explorer, editor, and the activity bar
//! all clone it. Mutations go through the VFS; async work (`prepare_html`,
//! `prepare_pdf`, folder pick, template fetch, first-visit help, ZIP) uses
//! `spawn_local` and the `loading` flag so two long actions cannot overlap.

mod actions;
mod commands;

use std::collections::HashSet;

use leptos::prelude::*;

use crate::fs::{join_path, parent_path, rewrite_prefix, Vfs};
use crate::persist::Session;

/// Whether an editor tab shows the source or a rendered preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabKind {
    /// Editable source.
    Edit,
    /// Rendered HTML / Markdown / PDF preview.
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
        let idx = active
            .as_ref()
            .and_then(|tab| tabs.iter().position(|open| open == tab));
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
