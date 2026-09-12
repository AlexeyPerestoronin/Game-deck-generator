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
use crate::conf;
use crate::export::{save_zip_bytes, vfs_to_zip, ZIP_FILENAME};
use crate::fs::VfsFs;
use crate::help;
use crate::load_folder::{
    install_files, install_folder, pick_and_read_files, pick_and_read_folder, PickFilesResult,
    PickResult,
};
use crate::persist::save_session;
use crate::template::install_new_game;

impl Workspace {
    /// Write the current snapshot to localStorage (manual Save).
    pub fn persist(&self) {
        match save_session(&self.snapshot()) {
            Ok(()) => self.status.set("Saved in this browser".into()),
            Err(err) => self.status.set(err),
        }
    }

    /// Copy bundled help into the VFS root if missing; preview if no tab is open.
    pub fn ensure_user_help(&self) {
        let open_preview = help::should_open_preview(self.tabs.get().len());
        if help::needs_install(&self.vfs.get()) {
            let mut vfs = self.vfs.get_untracked();
            match help::install_user_help(&mut vfs, help::bundled_help().to_string()) {
                Ok(()) => self.vfs.set(vfs),
                Err(err) => {
                    self.status.set(err);
                    return;
                }
            }
        }
        if open_preview {
            self.open_help_preview();
        }
    }

    fn open_help_preview(&self) {
        let path = conf::help::PATH.to_string();
        self.expand_ancestors(&path);
        self.selected.set(Some(path.clone()));
        self.open_tab(OpenTab {
            path,
            kind: TabKind::Preview,
        });
        self.status.set(format!("Opened {}", conf::help::PATH));
    }

    /// Empty the tree and persist that empty session.
    pub fn clear(&self) {
        self.vfs.set(crate::fs::Vfs::default());
        self.selected.set(None);
        self.tabs.set(Vec::new());
        self.active_tab.set(None);
        self.expanded.set(HashSet::new());
        self.status.set("Workspace cleared".into());
        let _ = save_session(&self.snapshot());
    }

    /// Pick local files and copy them into an existing workspace `folder`.
    pub fn load_files_into_folder(&self, folder: &str, warning: RwSignal<Option<String>>) {
        if self.loading.get() {
            return;
        }
        if !self.vfs.get().is_dir(folder) {
            warning.set(Some(format!("'{folder}' is not a folder")));
            return;
        }
        self.loading.set(true);
        self.status.set("Select file(s)…".into());
        let workspace = *self;
        let folder = folder.to_string();
        spawn_local(async move {
            match pick_and_read_files().await {
                PickFilesResult::Cancelled => {
                    workspace.status.set(String::new());
                }
                PickFilesResult::Rejected(reason) => {
                    warning.set(Some(reason));
                    workspace.status.set(String::new());
                }
                PickFilesResult::Ready { files } => {
                    let mut vfs = workspace.vfs.get_untracked();
                    match install_files(&mut vfs, &folder, &files) {
                        Ok(n) => {
                            workspace.vfs.set(vfs);
                            workspace.expand_ancestors(&folder);
                            workspace.selected.set(Some(folder.clone()));
                            workspace
                                .status
                                .set(format!("Loaded {n} file(s) into {folder}"));
                        }
                        Err(err) => workspace.status.set(err),
                    }
                }
            }
            workspace.loading.set(false);
        });
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

    /// Fetch the `new-game` template from GitHub and install it under `games/`.
    pub fn add_new_game(&self, warning: RwSignal<Option<String>>) {
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
                    workspace
                        .status
                        .set(format!("Added {path} from {}", installed.source));
                }
                Err(err) => {
                    warning.set(Some(err));
                    workspace.status.set(String::new());
                }
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
}

fn take_vfs(fs: Arc<VfsFs>) -> crate::fs::Vfs {
    match Arc::try_unwrap(fs) {
        Ok(inner) => inner.into_vfs(),
        Err(arc) => arc.clone_vfs(),
    }
}
