//! Find the repository-root `conf.json5` without baking a path into the binary.

use std::env;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::fs::FileSystem;

const CONF_FILE_NAME: &str = "conf.json5";
const CONF_PATH_ENV: &str = "DECK_GEN_CONF";

pub fn find_conf_file(fs: &dyn FileSystem) -> Result<PathBuf> {
    if let Ok(explicit) = env::var(CONF_PATH_ENV) {
        let path = PathBuf::from(explicit);
        if fs.is_file(&path) {
            return Ok(path);
        }
        return Err(Error::file(
            &path,
            format!("{CONF_PATH_ENV} does not point to a file"),
        ));
    }

    for start in fs.search_roots() {
        if let Some(found) = walk_parents_for_root_conf(fs, start) {
            return Ok(found);
        }
    }

    Err(Error::msg(format!(
        "Root {CONF_FILE_NAME} not found. Run from the repository, place the binary next to {CONF_FILE_NAME}, or set {CONF_PATH_ENV}"
    )))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn os_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
        }
    }
    roots
}

fn walk_parents_for_root_conf(fs: &dyn FileSystem, start: PathBuf) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        let candidate = dir.join(CONF_FILE_NAME);
        if fs.is_file(&candidate) && looks_like_root_conf(fs, &candidate) {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Root conf is the one that points at the games folder. Game and games-root
/// conf files share the same filename, so we must not stop at the first hit.
fn looks_like_root_conf(fs: &dyn FileSystem, path: &Path) -> bool {
    let Ok(text) = fs.read_to_string(path) else {
        return false;
    };
    let Ok(value) = json5::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    value.get("games_root").is_some()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn canonicalize_or_abs(path: &Path) -> PathBuf {
    let raw = path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    });
    strip_windows_verbatim_prefix(raw)
}

#[cfg(not(target_arch = "wasm32"))]
fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path
    }
}
