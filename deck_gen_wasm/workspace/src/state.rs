//! Reactive workspace: tree, selection, tabs, and the actions the UI triggers.
//!
//! [`Workspace`] is a cheap `Copy` handle to Leptos signals (VFS, selection,
//! multi-selection, copy plan, expanded folders, tabs, split preview, status,
//! loading, progress). Explorer, editor, and the activity bar all clone it.
//! Mutations go through the VFS; async work (`prepare_html`, `prepare_pdf`,
//! folder pick, template fetch, ZIP) uses `spawn_local` and the `loading` flag
//! so two long actions cannot overlap. `progress` is a 0..=100 hint for the
//! activity-bar ray and is not part of [`Session`].

use std::collections::HashSet;

use leptos::prelude::*;

use deck_gen_wasm_fs::{
    join_path, parent_path, retain_not_under, rewrite_prefix, rewrite_set, Vfs,
};
use deck_gen_wasm_persist::Session;

/// Whether an editor tab shows the source or a rendered preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabKind {
    /// Editable source.
    Edit,
    /// Rendered HTML / Markdown / PDF / image preview.
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
    /// Primary explorer selection (last clicked path, used by create/tabs).
    pub selected: RwSignal<Option<String>>,
    /// All explorer paths in the current multi-selection.
    pub multi_selected: RwSignal<HashSet<String>>,
    /// Paths marked for copy (in-app plan, not the OS clipboard).
    pub copy_planned: RwSignal<HashSet<String>>,
    /// Open editor tabs, left to right (edit tabs only while split is on).
    pub tabs: RwSignal<Vec<OpenTab>>,
    /// Focused tab in the main / left pane, if any.
    pub active_tab: RwSignal<Option<OpenTab>>,
    /// When true, the editor is split: files on the left, previews on the right.
    pub split_preview: RwSignal<bool>,
    /// Preview tabs in the right pane (empty while split is off).
    pub preview_tabs: RwSignal<Vec<OpenTab>>,
    /// Focused tab in the right preview pane, if any.
    pub active_preview_tab: RwSignal<Option<OpenTab>>,
    /// Directories currently expanded in the tree.
    pub expanded: RwSignal<HashSet<String>>,
    /// Footer status line.
    pub status: RwSignal<String>,
    /// True while an async action (load / template / HTML / PDF / ZIP) is running.
    pub loading: RwSignal<bool>,
    /// 0..=100 fill of the progress ray while [`Self::loading`] is true. Not persisted.
    pub progress: RwSignal<f32>,
    /// Focused editor text not yet written into [`Self::vfs`].
    pub draft: RwSignal<Option<(String, String)>>,
}

impl Workspace {
    /// Restore signals from a localStorage snapshot (or an empty default).
    pub fn from_session(session: Session) -> Self {
        let expanded = session.expanded_set();
        let (tabs, active_tab) = initial_tabs(&session.vfs, &session.selected);
        let multi_selected = session.selected.iter().cloned().collect();
        Self {
            vfs: RwSignal::new(session.vfs),
            selected: RwSignal::new(session.selected),
            multi_selected: RwSignal::new(multi_selected),
            copy_planned: RwSignal::new(HashSet::new()),
            tabs: RwSignal::new(tabs),
            active_tab: RwSignal::new(active_tab),
            split_preview: RwSignal::new(false),
            preview_tabs: RwSignal::new(Vec::new()),
            active_preview_tab: RwSignal::new(None),
            expanded: RwSignal::new(expanded),
            status: RwSignal::new(String::new()),
            loading: RwSignal::new(false),
            progress: RwSignal::new(0.0),
            draft: RwSignal::new(None),
        }
    }

    /// Remember the focused file body without notifying VFS subscribers.
    pub fn set_draft(&self, path: String, content: String) {
        self.draft
            .update_untracked(|slot| *slot = Some((path, content)));
    }

    /// Write the editor draft into VFS if it differs from the stored file.
    pub fn flush_draft(&self) {
        let Some((path, content)) = self.draft.get_untracked() else {
            return;
        };
        self.draft.update_untracked(|slot| *slot = None);
        let unchanged = self
            .vfs
            .with_untracked(|vfs| vfs.read_file(&path) == Some(content.as_str()));
        if unchanged {
            return;
        }
        self.vfs.update(|vfs| {
            let _ = vfs.write_file(&path, content);
        });
    }

    /// True when the draft for `path` is newer than the VFS file.
    pub fn draft_is_ahead(&self, path: &str) -> bool {
        self.draft.with_untracked(|draft| match draft {
            Some((p, c)) if p == path => self
                .vfs
                .with_untracked(|vfs| vfs.read_file(p) != Some(c.as_str())),
            _ => false,
        })
    }

    /// Serializable snapshot for localStorage (binaries omitted; they go to IndexedDB).
    pub fn snapshot(&self) -> Session {
        self.flush_draft();
        self.vfs
            .with(|vfs| Session::from_workspace(vfs, self.selected.get(), &self.expanded.get()))
    }

    /// Select `path`; folders also toggle expansion. Files open an edit tab.
    pub fn select(&self, path: String, is_dir: bool) {
        self.set_primary_selection(Some(path.clone()));
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

    /// Ctrl+click: add or remove `path` from the multi-selection.
    pub fn toggle_select(&self, path: String) {
        self.status.set(String::new());
        self.multi_selected.update(|set| {
            if !set.remove(&path) {
                set.insert(path.clone());
            }
        });
        let set = self.multi_selected.get();
        if set.contains(&path) {
            self.selected.set(Some(path));
        } else {
            self.selected.set(set.iter().next().cloned());
        }
    }

    pub(crate) fn set_primary_selection(&self, path: Option<String>) {
        match &path {
            Some(path) => self.multi_selected.set(HashSet::from([path.clone()])),
            None => self.multi_selected.set(HashSet::new()),
        }
        self.selected.set(path);
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
                self.set_primary_selection(Some(path.clone()));
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
                self.set_primary_selection(Some(path));
                self.status.set(String::new());
            }
            Some(Err(err)) => self.status.set(err),
            None => self.status.set("Could not update workspace".into()),
        }
    }

    pub(crate) fn forget_path(&self, path: &str) {
        let gone = |current: &str| deck_gen_wasm_fs::path_is_or_under(current, path);
        self.selected.update(|selected| {
            if selected.as_ref().is_some_and(|current| gone(current)) {
                *selected = None;
            }
        });
        self.multi_selected
            .update(|set| retain_not_under(set, path));
        self.copy_planned.update(|set| retain_not_under(set, path));
        self.expanded.update(|set| retain_not_under(set, path));
        let (remaining, next) =
            crate::split::forget_tabs(self.tabs.get(), self.active_tab.get(), &gone);
        let (preview_remaining, preview_next) = crate::split::forget_tabs(
            self.preview_tabs.get(),
            self.active_preview_tab.get(),
            &gone,
        );
        self.tabs.set(remaining);
        self.active_tab.set(next.clone());
        self.preview_tabs.set(preview_remaining);
        self.active_preview_tab.set(preview_next);
        if let Some(next) = next {
            self.set_primary_selection(Some(next.path));
        }
    }

    pub(crate) fn rewrite_paths(&self, old: &str, new: &str) {
        self.selected.update(|selected| {
            if let Some(current) = selected.as_ref() {
                *selected = Some(rewrite_prefix(current, old, new));
            }
        });
        self.multi_selected.update(|set| rewrite_set(set, old, new));
        self.copy_planned.update(|set| rewrite_set(set, old, new));
        self.expanded.update(|set| rewrite_set(set, old, new));
        self.tabs.update(|tabs| {
            for tab in tabs.iter_mut() {
                tab.path = rewrite_prefix(&tab.path, old, new);
            }
        });
        self.preview_tabs.update(|tabs| {
            for tab in tabs.iter_mut() {
                tab.path = rewrite_prefix(&tab.path, old, new);
            }
        });
        self.active_tab.update(|active| {
            if let Some(tab) = active {
                tab.path = rewrite_prefix(&tab.path, old, new);
            }
        });
        self.active_preview_tab.update(|active| {
            if let Some(tab) = active {
                tab.path = rewrite_prefix(&tab.path, old, new);
            }
        });
    }

    fn creation_parent(&self) -> String {
        match self.selected.get() {
            Some(path) if self.vfs.with(|vfs| vfs.is_dir(&path)) => path,
            Some(path) => parent_path(&path).to_string(),
            None => String::new(),
        }
    }

    pub(crate) fn expand_ancestors(&self, path: &str) {
        self.expanded.update(|set| {
            let mut current = path;
            while !current.is_empty() {
                set.insert(current.to_string());
                current = parent_path(current);
            }
        });
    }

    /// True if an async action may start; sets `loading`, `progress = 0`, and the status line.
    pub(crate) fn try_begin_async(&self, status: impl Into<String>) -> bool {
        if self.loading.get() {
            return false;
        }
        self.loading.set(true);
        self.progress.set(0.0);
        self.status.set(status.into());
        true
    }

    pub(crate) fn finish_async(&self) {
        self.loading.set(false);
    }

    /// Callback that writes [`Self::progress`]. Does not yield; callers await a frame between blocks.
    pub(crate) fn progress_handle(&self) -> deck_gen_wasm_progress::Progress {
        let progress = self.progress;
        deck_gen_wasm_progress::Progress::new(move |pct| progress.set(pct))
    }

    pub(crate) fn take_pick<T>(
        &self,
        warning: RwSignal<Option<String>>,
        result: deck_gen_wasm_import::PickOutcome<T>,
    ) -> Option<T> {
        use deck_gen_wasm_import::PickOutcome;
        match result {
            PickOutcome::Cancelled => {
                self.status.set(String::new());
                None
            }
            PickOutcome::Rejected(reason) => {
                warning.set(Some(reason));
                self.status.set(String::new());
                None
            }
            PickOutcome::Ready(value) => Some(value),
        }
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

pub(crate) fn ask_name(message: &str) -> Option<String> {
    let window = web_sys::window()?;
    window
        .prompt_with_message(message)
        .ok()
        .flatten()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}
