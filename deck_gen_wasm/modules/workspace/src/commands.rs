//! Explorer context-menu commands on a file or folder path.
//!
//! `copy` only updates the in-app copy plan. `past` clones planned VFS nodes
//! into the target folder and then clears the plan.

use std::collections::HashSet;

use leptos::prelude::*;

use super::copy_plan::{apply_copy_command, top_level_paths};
use super::{OpenTab, TabKind, Workspace};
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

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
            other => {
                let tmpl = locale::localize(keys::STATUS_UNKNOWN_COMMAND);
                self.status.set(tmpl.replace("{other}", other));
            }
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
            self.status.set(locale::localize(keys::STATUS_NOTHING_TO_PASTE));
            return;
        }
        self.flush_draft();
        let sources = top_level_paths(&planned);
        match self
            .vfs
            .try_update(|vfs| vfs.copy_entries_into(&sources, dest))
        {
            Some(Ok(n)) => {
                self.copy_planned.set(HashSet::new());
                self.expand_ancestors(dest);
                let tmpl = locale::localize(keys::STATUS_PASTED);
                self.status.set(tmpl.replace("{n}", &n.to_string()).replace("{dest}", dest));
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set(locale::localize(keys::STATUS_COULD_NOT_UPDATE)),
        }
    }

    fn delete_entry(&self, path: &str) {
        self.flush_draft();
        match self.vfs.try_update(|vfs| vfs.remove(path)) {
            Some(Ok(())) => {
                self.forget_path(path);
                let tmpl = locale::localize(keys::STATUS_DELETED);
                self.status.set(tmpl.replace("{path}", path));
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set(locale::localize(keys::STATUS_COULD_NOT_UPDATE)),
        }
    }

    fn rename_entry(&self, path: &str) {
        let Some(name) = crate::state::ask_name(&locale::localize(keys::PROMPT_NEW_NAME)) else {
            return;
        };
        self.flush_draft();
        match self.vfs.try_update(|vfs| vfs.rename(path, &name)) {
            Some(Ok(new_path)) => {
                self.rewrite_paths(path, &new_path);
                let tmpl = locale::localize(keys::STATUS_RENAMED);
                self.status.set(tmpl.replace("{new_path}", &new_path));
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set(locale::localize(keys::STATUS_COULD_NOT_UPDATE)),
        }
    }
}
