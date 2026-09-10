//! Find `conf.json5` without baking a repository path into the binary.

use std::env;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

const CONF_FILE_NAME: &str = "conf.json5";
const CONF_PATH_ENV: &str = "DECK_GEN_CONF";

pub fn find_conf_file() -> Result<PathBuf> {
    if let Ok(explicit) = env::var(CONF_PATH_ENV) {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Ok(path);
        }
        return Err(Error::file(
            &path,
            format!("{CONF_PATH_ENV} does not point to a file"),
        ));
    }

    for start in search_roots() {
        if let Some(found) = walk_parents_for_conf(start) {
            return Ok(found);
        }
    }

    Err(Error::msg(format!(
        "{CONF_FILE_NAME} not found. Run from the repository, place the binary next to {CONF_FILE_NAME}, or set {CONF_PATH_ENV}"
    )))
}

fn search_roots() -> Vec<PathBuf> {
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

fn walk_parents_for_conf(start: PathBuf) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        let candidate = dir.join(CONF_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

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

fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path
    }
}
