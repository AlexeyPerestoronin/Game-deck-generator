//! Long-running workspace actions: persist, help, import, template, prepare, ZIP.
//!
//! These methods share the [`Workspace`](super::Workspace) `loading` flag so
//! the activity bar can disable overlapping work. Async paths yield once before
//! `prepare_*` so Leptos can paint the status line.

use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::{OpenTab, TabKind, Workspace};
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_export::{save_zip_bytes, vfs_to_zip, ZIP_FILENAME};
use crate::vfs_fs::VfsFs;
use deck_gen_wasm_template as help;
use deck_gen_wasm_import::{
    install_files, install_folder, pick_and_read_files, pick_and_read_folder, PickedFiles,
    PickedFolder,
};
use deck_gen_wasm_persist::{save_binaries, save_session};
use deck_gen_wasm_template::install_new_game;

impl Workspace {
    /// Write the text snapshot to localStorage and binaries to IndexedDB.
    pub fn persist(&self) {
        match save_session(&self.snapshot()) {
            Ok(()) => {
                let vfs = self.vfs.with(|vfs| vfs.clone());
                let status = self.status;
                spawn_local(async move {
                    match save_binaries(&vfs).await {
                        Ok(()) => status.set("Saved in this browser".into()),
                        Err(err) => status.set(err),
                    }
                });
            }
            Err(err) => self.status.set(err),
        }
    }

    /// Copy bundled help into the VFS root if missing; preview if no tab is open.
    pub fn ensure_user_help(&self) {
        let open_preview = self.tabs.with(|tabs| tabs.is_empty());
        let result = self.vfs.try_update(|vfs| {
            if help::needs_install(vfs) {
                help::install_user_help(vfs)
            } else {
                Ok(())
            }
        });
        match result {
            Some(Err(err)) => {
                self.status.set(err);
                return;
            }
            None => {
                self.status.set("Could not update workspace".into());
                return;
            }
            Some(Ok(())) => {}
        }
        if open_preview {
            self.open_help_preview();
        }
    }

    fn open_help_preview(&self) {
        let path = conf::help::PATH.to_string();
        self.expand_ancestors(&path);
        self.set_primary_selection(Some(path.clone()));
        self.open_tab(OpenTab {
            path,
            kind: TabKind::Preview,
        });
        self.status.set(format!("Opened {}", conf::help::PATH));
    }

    /// Empty the tree and persist that empty session.
    pub fn clear(&self) {
        self.draft.set(None);
        self.vfs.set(deck_gen_wasm_fs::Vfs::default());
        self.set_primary_selection(None);
        self.copy_planned.set(HashSet::new());
        self.tabs.set(Vec::new());
        self.active_tab.set(None);
        self.preview_tabs.set(Vec::new());
        self.active_preview_tab.set(None);
        self.expanded.set(HashSet::new());
        self.status.set("Workspace cleared".into());
        let _ = save_session(&self.snapshot());
        spawn_local(async {
            let _ = save_binaries(&deck_gen_wasm_fs::Vfs::default()).await;
        });
    }

    /// Pick local files and copy them into an existing workspace `folder`.
    pub fn load_files_into_folder(&self, folder: &str, warning: RwSignal<Option<String>>) {
        if self.vfs.with(|vfs| !vfs.is_dir(folder)) {
            warning.set(Some(format!("'{folder}' is not a folder")));
            return;
        }
        if !self.try_begin_async("Select file(s)…") {
            return;
        }
        let workspace = *self;
        let folder = folder.to_string();
        spawn_local(async move {
            if let Some(PickedFiles { files }) =
                workspace.take_pick(warning, pick_and_read_files().await)
            {
                workspace.flush_draft();
                let mut vfs = workspace.vfs.get_untracked();
                match install_files(&mut vfs, &folder, &files) {
                    Ok(n) => {
                        workspace.vfs.set(vfs);
                        workspace.expand_ancestors(&folder);
                        workspace.set_primary_selection(Some(folder.clone()));
                        workspace
                            .status
                            .set(format!("Loaded {n} file(s) into {folder}"));
                    }
                    Err(err) => workspace.status.set(err),
                }
            }
            workspace.finish_async();
        });
    }

    /// Pick a local folder and copy it under `games/` with a unique name.
    pub fn load_game_from_disk(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async("Select a folder…") {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            if let Some(PickedFolder { name, files, dirs }) =
                workspace.take_pick(warning, pick_and_read_folder().await)
            {
                workspace.status.set("Loading folder…".into());
                workspace.flush_draft();
                let mut vfs = workspace.vfs.get_untracked();
                match install_folder(&mut vfs, &name, &dirs, &files) {
                    Ok(folder) => {
                        workspace.vfs.set(vfs);
                        let path = format!("games/{folder}");
                        workspace.expand_ancestors(&path);
                        workspace.set_primary_selection(Some(path.clone()));
                        workspace.status.set(format!("Loaded {path}"));
                    }
                    Err(err) => workspace.status.set(err),
                }
            }
            workspace.finish_async();
        });
    }

    /// Run [`deck_gen::prepare_html`] on the VFS after a 0ms yield so the UI can paint.
    pub fn prepare_html(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async("Preparing HTML…") {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            workspace.flush_draft();
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
            workspace.finish_async();
        });
    }

    /// Run [`deck_gen::prepare_pdf`] with the browser engine after a 0ms yield.
    pub fn prepare_pdf(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async("Preparing PDF…") {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            workspace.flush_draft();
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
            workspace.finish_async();
        });
    }

    /// Fetch the `new-game` template from GitHub and install it under `games/`.
    pub fn add_new_game(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async("Loading new-game template…") {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            workspace.flush_draft();
            let mut vfs = workspace.vfs.get_untracked();
            let result = install_new_game(&mut vfs).await;
            match result {
                Ok(installed) => {
                    workspace.vfs.set(vfs);
                    let path = format!("games/{}", installed.folder);
                    workspace.expand_ancestors(&path);
                    workspace.set_primary_selection(Some(path.clone()));
                    workspace
                        .status
                        .set(format!("Added {path} from {}", installed.source));
                }
                Err(err) => {
                    warning.set(Some(err));
                    workspace.status.set(String::new());
                }
            }
            workspace.finish_async();
        });
    }

    /// Encode the tree as ZIP and offer it to the browser.
    pub fn download(&self) {
        self.flush_draft();
        match self.vfs.with(vfs_to_zip) {
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
}

fn take_vfs(fs: Arc<VfsFs>) -> deck_gen_wasm_fs::Vfs {
    match Arc::try_unwrap(fs) {
        Ok(inner) => inner.into_vfs(),
        Err(arc) => arc.clone_vfs(),
    }
}


