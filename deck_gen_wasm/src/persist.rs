//! Browser localStorage snapshot of the in-memory workspace.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::fs::Vfs;

const STORAGE_KEY: &str = "deck_gen_wasm.session";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Session {
    pub vfs: Vfs,
    pub selected: Option<String>,
    pub expanded: Vec<String>,
}

impl Session {
    pub fn from_workspace(vfs: Vfs, selected: Option<String>, expanded: &HashSet<String>) -> Self {
        let mut dirs: Vec<String> = expanded.iter().cloned().collect();
        dirs.sort();
        Self {
            vfs,
            selected,
            expanded: dirs,
        }
    }

    pub fn expanded_set(&self) -> HashSet<String> {
        self.expanded.iter().cloned().collect()
    }
}

pub fn load_session() -> Option<Session> {
    let raw = local_storage()?.get_item(STORAGE_KEY).ok().flatten()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_session(session: &Session) -> Result<(), String> {
    let raw = serde_json::to_string(session).map_err(|err| err.to_string())?;
    local_storage()
        .ok_or_else(|| "localStorage is not available".to_string())?
        .set_item(STORAGE_KEY, &raw)
        .map_err(|_| "Could not write to localStorage".to_string())
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}
