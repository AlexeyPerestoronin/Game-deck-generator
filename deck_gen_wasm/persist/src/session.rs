//! Browser snapshot of the in-memory workspace.
//!
//! Text files, selection, and expanded folders live in localStorage as JSON.
//! PDF and image bytes are too large for that quota, so they go to IndexedDB
//! ([`binaries`]). Load is best-effort; a corrupt or missing key starts an
//! empty workspace. Save is triggered from the activity bar and from a Leptos
//! effect on every signal change (binaries only when their fingerprint moves).

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::{kind, Vfs};

/// Serializable workspace snapshot stored under `deck_gen_wasm.session`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Session {
    /// Full in-memory tree (binaries stripped before write).
    pub vfs: Vfs,
    /// Selected explorer path, if any.
    pub selected: Option<String>,
    /// Expanded directory paths (sorted on write).
    pub expanded: Vec<String>,
}

impl Session {
    /// Build a snapshot; expanded dirs are sorted for stable JSON.
    /// Files that are persisted as binary (images, pdfs) are omitted from the
    /// tree (their data lives in IndexedDB). This is decided using `kind`.
    pub fn from_workspace(vfs: &Vfs, selected: Option<String>, expanded: &HashSet<String>) -> Self {
        let mut dirs: Vec<String> = expanded.iter().cloned().collect();
        dirs.sort();

        let mut text_vfs = Vfs::default();
        vfs.visit_entries(|path, data| {
            if let Some(bytes) = data {
                if is_persisted_as_binary(path) {
                    // omit the entry entirely (matches previous Binary stripping)
                    return;
                }
                // text content
                if let Ok(text) = std::str::from_utf8(bytes) {
                    let _ = text_vfs.put_file(path, text.to_string());
                }
            } else {
                let _ = text_vfs.mkdir(path);
            }
        });

        Self {
            vfs: text_vfs,
            selected,
            expanded: dirs,
        }
    }

    /// Expanded paths as a set for the explorer.
    pub fn expanded_set(&self) -> HashSet<String> {
        self.expanded.iter().cloned().collect()
    }
}

/// Read and deserialize the session, or `None` if missing/invalid.
pub fn load_session() -> Option<Session> {
    let raw = local_storage()?
        .get_item(conf::session::STORAGE_KEY)
        .ok()
        .flatten()?;
    serde_json::from_str(&raw).ok()
}

/// Serialize `session` to localStorage.
pub fn save_session(session: &Session) -> Result<(), String> {
    let raw = serde_json::to_string(session).map_err(|err| err.to_string())?;
    local_storage()
        .ok_or_else(|| "localStorage is not available".to_string())?
        .set_item(conf::session::STORAGE_KEY, &raw)
        .map_err(|_| "Could not write to localStorage".to_string())
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn is_persisted_as_binary(path: &str) -> bool {
    matches!(kind::kind_of(path), kind::FileKind::Image | kind::FileKind::Pdf)
}
