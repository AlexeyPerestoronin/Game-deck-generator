//! Explorer context-menu commands on a file or folder path.

use leptos::prelude::*;

use super::{OpenTab, TabKind, Workspace};

impl Workspace {
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
        let Some(name) = super::ask_name("New name") else {
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
}
