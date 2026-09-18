//! Long-running workspace actions: persist, help, import, template, prepare, ZIP.
//!
//! These methods share the [`Workspace`](super::Workspace) `loading` flag so
//! the activity bar can disable overlapping work. Long paths run on the
//! browser event loop via `spawn_local` (WASM has no OS threads) and report
//! 0..=100 through `progress_*` macros; each `set` yields a frame so the ray
//! can paint.

use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::{OpenTab, TabKind, Workspace};
use crate::vfs_fs::VfsFs;
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_export::{save_zip_bytes, vfs_to_zip, ZIP_FILENAME};
use deck_gen_wasm_import::{
    install_files, install_folder, pick_and_read_files, pick_and_read_folder, PickedFiles,
    PickedFolder,
};
use deck_gen_wasm_persist::{save_binaries, save_session};
use progress_viewer::{progress_block, progress_wrapper};
use deck_gen_wasm_template as help;
use deck_gen_wasm_template::{install_game, install_new_game};

impl Workspace {
    /// Write the text snapshot to localStorage and binaries to IndexedDB.
    pub fn persist(&self) {
        match save_session(&self.snapshot()) {
            Ok(()) => {
                let vfs = self.vfs.with(|vfs| vfs.clone());
                let status = self.status;
                spawn_local(async move {
                    match save_binaries(&vfs).await {
                        Ok(()) => status.set(locale::localize(keys::STATUS_SAVED)),
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
                self.status.set(locale::localize(keys::STATUS_COULD_NOT_UPDATE));
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
        let tmpl = locale::localize(keys::STATUS_OPENED);
        self.status.set(tmpl.replace("{path}", conf::help::PATH));
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
        self.status.set(locale::localize(keys::STATUS_WORKSPACE_CLEARED));
        let _ = save_session(&self.snapshot());
        spawn_local(async {
            let _ = save_binaries(&deck_gen_wasm_fs::Vfs::default()).await;
        });
    }

    /// Pick local files and copy them into an existing workspace `folder`.
    pub fn load_files_into_folder(&self, folder: &str, warning: RwSignal<Option<String>>) {
        if self.vfs.with(|vfs| !vfs.is_dir(folder)) {
            let tmpl = locale::localize(keys::STATUS_NOT_A_FOLDER);
            warning.set(Some(tmpl.replace("{path}", folder)));
            return;
        }
        if !self.try_begin_async(locale::localize(keys::STATUS_SELECT_FILES)) {
            return;
        }
        let workspace = *self;
        let folder = folder.to_string();
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                let picked = progress_block!(progress, 0.0, 30.0, {
                    workspace.take_pick(warning, pick_and_read_files().await)
                });
                if let Some(PickedFiles { files }) = picked {
                    let installed = progress_block!(progress, 30.0, 90.0, {
                        workspace.flush_draft();
                        let mut vfs = workspace.vfs.get_untracked();
                        let subprocess = progress.new_subprocess(40.0, 90.0);
                        let result = install_files(&mut vfs, &folder, &files, subprocess).await;
                        (vfs, result)
                    });
                    progress_block!(progress, 90.0, 100.0, {
                        match installed {
                            (vfs, Ok(n)) => {
                                workspace.vfs.set(vfs);
                                workspace.expand_ancestors(&folder);
                                workspace.set_primary_selection(Some(folder.clone()));
                                let tmpl = locale::localize(keys::STATUS_LOADED_N_INTO);
                                workspace
                                    .status
                                    .set(tmpl.replace("{n}", &n.to_string()).replace("{folder}", &folder));
                            }
                            (_, Err(err)) => workspace.status.set(err),
                        }
                    });
                }
            });
            workspace.finish_async();
        });
    }

    /// Pick a local folder and copy it under `games/` with a unique name.
    pub fn load_game_from_disk(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async(locale::localize(keys::STATUS_SELECT_FOLDER)) {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                let picked = progress_block!(progress, 0.0, 30.0, {
                    workspace.take_pick(warning, pick_and_read_folder().await)
                });
                if let Some(PickedFolder { name, files, dirs }) = picked {
                    workspace.status.set(locale::localize(keys::STATUS_LOADING_FOLDER));
                    let installed = progress_block!(progress, 30.0, 90.0, {
                        workspace.flush_draft();
                        let mut vfs = workspace.vfs.get_untracked();
                        let subprocess = progress.new_subprocess(40.0, 90.0);
                        let result =
                            install_folder(&mut vfs, &name, &dirs, &files, subprocess).await;
                        (vfs, result)
                    });
                    progress_block!(progress, 90.0, 100.0, {
                        match installed {
                            (vfs, Ok(folder)) => {
                                workspace.vfs.set(vfs);
                                let path = format!("games/{folder}");
                                workspace.expand_ancestors(&path);
                                workspace.set_primary_selection(Some(path.clone()));
                                let tmpl = locale::localize(keys::STATUS_LOADED);
                                workspace.status.set(tmpl.replace("{path}", &path));
                            }
                            (_, Err(err)) => workspace.status.set(err),
                        }
                    });
                }
            });
            workspace.finish_async();
        });
    }

    /// Run [`deck_gen::prepare_html`] on the VFS after a 0ms yield so the UI can paint.
    pub fn prepare_html(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async(locale::localize(keys::STATUS_PREPARING_HTML)) {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                progress_block!(progress, 0.0, 10.0, {});
                let prepared = progress_block!(progress, 10.0, 90.0, {
                    workspace.flush_draft();
                    let fs = Arc::new(VfsFs::new(workspace.vfs.get_untracked()));
                    let sub = progress.new_subprocess(10.0, 90.0);
                    let result = deck_gen::prepare_html(fs.clone(), &sub);
                    (fs, result)
                });
                progress_block!(progress, 90.0, 100.0, {
                    let (fs, result) = prepared;
                    match result {
                        Ok(n) => {
                            workspace.vfs.set(take_vfs(fs));
                            let tmpl = locale::localize(keys::STATUS_PREPARED_HTML);
                            workspace.status.set(tmpl.replace("{n}", &n.to_string()));
                        }
                        Err(err) => {
                            warning.set(Some(err.to_string()));
                            workspace.status.set(String::new());
                        }
                    }
                });
            });
            workspace.finish_async();
        });
    }

    /// Run [`deck_gen::prepare_pdf`] with the browser engine after a 0ms yield.
    pub fn prepare_pdf(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async(locale::localize(keys::STATUS_PREPARING_PDF)) {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                progress_block!(progress, 0.0, 10.0, {});
                let prepared = progress_block!(progress, 10.0, 90.0, {
                    workspace.flush_draft();
                    let fs = Arc::new(VfsFs::new(workspace.vfs.get_untracked()));
                    let engine = prepare_pdf_web::WebPdfEngine;
                    let sub = progress.new_subprocess(10.0, 90.0);
                    let result = deck_gen::prepare_pdf(fs.clone(), &engine, &sub).await;
                    (fs, result)
                });
                progress_block!(progress, 90.0, 100.0, {
                    let (fs, result) = prepared;
                    match result {
                        Ok(n) => {
                            workspace.vfs.set(take_vfs(fs));
                            let tmpl = locale::localize(keys::STATUS_PREPARED_PDF);
                            workspace.status.set(tmpl.replace("{n}", &n.to_string()));
                        }
                        Err(err) => {
                            warning.set(Some(err.to_string()));
                            workspace.status.set(String::new());
                        }
                    }
                });
            });
            workspace.finish_async();
        });
    }

    /// Fetch the `new-game` template from GitHub and install it under `games/`.
    pub fn add_new_game(&self, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async(locale::localize(keys::STATUS_LOADING_TEMPLATE)) {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                progress_block!(progress, 0.0, 10.0, {
                    workspace.flush_draft();
                });
                let installed = progress_block!(progress, 10.0, 90.0, {
                    let mut vfs = workspace.vfs.get_untracked();
                    let subprocess = progress.new_subprocess(10.0, 90.0);
                    let result = install_new_game(&mut vfs, subprocess).await;
                    (vfs, result)
                });
                progress_block!(progress, 90.0, 100.0, {
                    match installed {
                        (vfs, Ok(installed)) => {
                            workspace.vfs.set(vfs);
                            let path = format!("games/{}", installed.folder);
                            workspace.expand_ancestors(&path);
                            workspace.set_primary_selection(Some(path.clone()));
                            let tmpl = locale::localize(keys::STATUS_ADDED);
                            workspace
                                .status
                                .set(tmpl.replace("{path}", &path).replace("{source}", &installed.source));
                        }
                        (_, Err(err)) => {
                            warning.set(Some(err));
                            workspace.status.set(String::new());
                        }
                    }
                });
            });
            workspace.finish_async();
        });
    }

    /// Fetch a specific game folder (e.g. "monopoly-2.0") from GitHub and install under unique name in `games/`.
    pub fn add_game_from_github(&self, source_game: &str, warning: RwSignal<Option<String>>) {
        if !self.try_begin_async(locale::localize(keys::STATUS_LOADING_TEMPLATE)) {
            return;
        }
        let workspace = *self;
        let source_game = source_game.to_string();
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                progress_block!(progress, 0.0, 10.0, {
                    workspace.flush_draft();
                });
                let installed = progress_block!(progress, 10.0, 90.0, {
                    let mut vfs = workspace.vfs.get_untracked();
                    let subprocess = progress.new_subprocess(10.0, 90.0);
                    let result = install_game(&source_game, &mut vfs, subprocess).await;
                    (vfs, result)
                });
                progress_block!(progress, 90.0, 100.0, {
                    match installed {
                        (vfs, Ok(installed)) => {
                            workspace.vfs.set(vfs);
                            let path = format!("games/{}", installed.folder);
                            workspace.expand_ancestors(&path);
                            workspace.set_primary_selection(Some(path.clone()));
                            let tmpl = locale::localize(keys::STATUS_ADDED);
                            workspace
                                .status
                                .set(tmpl.replace("{path}", &path).replace("{source}", &installed.source));
                        }
                        (_, Err(err)) => {
                            warning.set(Some(err));
                            workspace.status.set(String::new());
                        }
                    }
                });
            });
            workspace.finish_async();
        });
    }

    /// Encode the tree as ZIP and offer it to the browser.
    pub fn download(&self) {
        if !self.try_begin_async(locale::localize(keys::STATUS_DOWNLOADING_ZIP)) {
            return;
        }
        let workspace = *self;
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                let encoded = progress_block!(progress, 0.0, 50.0, {
                    workspace.flush_draft();
                    let subprocess = progress.new_subprocess(0.0, 50.0);
                    let vfs = workspace.vfs.get_untracked();
                    vfs_to_zip(&vfs, subprocess).await
                });
                progress_block!(progress, 50.0, 100.0, {
                    match encoded {
                        Ok(bytes) => {
                            let subprocess = progress.new_subprocess(50.0, 100.0);
                            match save_zip_bytes(bytes, ZIP_FILENAME, subprocess).await {
                                Ok(()) => {
                                    let tmpl = locale::localize(keys::STATUS_DOWNLOADED);
                                    workspace.status.set(tmpl.replace("{filename}", ZIP_FILENAME))
                                }
                                Err(err) => workspace.status.set(err),
                            }
                        }
                        Err(err) => workspace.status.set(err),
                    }
                });
            });
            workspace.finish_async();
        });
    }
}

fn take_vfs(fs: Arc<VfsFs>) -> deck_gen_wasm_fs::Vfs {
    match Arc::try_unwrap(fs) {
        Ok(inner) => inner.into_vfs(),
        Err(arc) => arc.clone_vfs(),
    }
}
