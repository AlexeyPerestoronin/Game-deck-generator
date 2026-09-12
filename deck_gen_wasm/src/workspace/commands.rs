//! Explorer context-menu commands on a file or folder path.
//!
//! `copy` only updates the in-app copy plan. `past` clones planned VFS nodes
//! into the target folder and then clears the plan.

use std::collections::HashSet;

use leptos::prelude::*;

use super::copy_plan::{apply_copy_command, top_level_paths};
use super::{OpenTab, TabKind, Workspace};

impl Workspace {
    /// Context-menu command (`rename` / `delete` / `preview` / `load_files` / `copy` / `past`) on an explorer entry.
    pub fn run_entry_command(&self, id: &str, path: &str, warning: RwSignal<Option<String>>) {
        match id {
            "delete" => self.delete_entry(path),
            "rename" => self.rename_entry(path),
            "preview" => self.open_tab(OpenTab {
                path: path.to_string(),
                kind: TabKind::Preview,
            }),
            "load_files" => self.load_files_into_folder(path, warning),
            "copy" => self.mark_copy(path),
            "past" => self.paste_into(path),
            other => self.status.set(format!("Unknown command {other}")),
        }
    }

    fn mark_copy(&self, path: &str) {
        let next = apply_copy_command(path, &self.multi_selected.get(), &self.copy_planned.get());
        self.copy_planned.set(next);
        self.status.set(String::new());
    }

    fn paste_into(&self, dest: &str) {
        let planned = self.copy_planned.get();
        if planned.is_empty() {
            self.status.set("Nothing to paste".into());
            return;
        }
        let sources = top_level_paths(&planned);
        match self
            .vfs
            .try_update(|vfs| vfs.copy_entries_into(&sources, dest))
        {
            Some(Ok(n)) => {
                self.copy_planned.set(HashSet::new());
                self.expand_ancestors(dest);
                self.status.set(format!("Pasted {n} item(s) into {dest}"));
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set("Could not update workspace".into()),
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
