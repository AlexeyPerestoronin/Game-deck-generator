//! Find the repository-root `conf.json5` without baking a path into the binary.
//!
//! Search order: `DECK_GEN_CONF` if it points at a file, then walk parents of
//! each [`FileSystem::search_roots`] entry. Game folders also contain a
//! `conf.json5`, so a hit counts only when the file has `games_root`.

use std::env;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::fs::FileSystem;

pub(crate) const CONF_FILE_NAME: &str = "conf.json5";
const CONF_PATH_ENV: &str = "DECK_GEN_CONF";

/// Resolve the root conf path through `fs` (env override, then parent walk).
pub fn find_conf_file<F>(fs: &F) -> Result<PathBuf>
where
    F: FileSystem + ?Sized,
{
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

fn walk_parents_for_root_conf<F>(fs: &F, start: PathBuf) -> Option<PathBuf>
where
    F: FileSystem + ?Sized,
{
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
fn looks_like_root_conf<F>(fs: &F, path: &Path) -> bool
where
    F: FileSystem + ?Sized,
{
    let Ok(text) = fs.read_to_string(path) else {
        return false;
    };
    let Ok(value) = json5::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    value.get("games_root").is_some()
}
