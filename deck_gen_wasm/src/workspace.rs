//! Reactive workspace: tree, selection, and the actions the UI triggers.

use std::collections::HashSet;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::export::{save_zip_bytes, vfs_to_zip, ZIP_FILENAME};
use crate::fs::{join_path, parent_path, Vfs};
use crate::persist::{save_session, Session};

#[derive(Clone, Copy)]
pub struct Workspace {
    pub vfs: RwSignal<Vfs>,
    pub selected: RwSignal<Option<String>>,
    pub expanded: RwSignal<HashSet<String>>,
    pub status: RwSignal<String>,
}

impl Workspace {
    pub fn from_session(session: Session) -> Self {
        let expanded = session.expanded_set();
        Self {
            vfs: RwSignal::new(session.vfs),
            selected: RwSignal::new(session.selected),
            expanded: RwSignal::new(expanded),
            status: RwSignal::new(String::new()),
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

fn ask_name(message: &str) -> Option<String> {
    let window = web_sys::window()?;
    window
        .prompt_with_message(message)
        .ok()
        .flatten()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}
