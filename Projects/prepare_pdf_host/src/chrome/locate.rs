//! Find a Chrome/Chromium executable from caller-supplied search rules.

use std::path::{Path, PathBuf};

use headless_chrome::browser::default_executable;

use crate::error::{Error, Result};

pub struct ChromeLocator {
    pub env_vars: Vec<String>,
    pub executables: Vec<PathBuf>,
    pub home_relative: Vec<PathBuf>,
}

pub fn find_chrome(locator: &ChromeLocator) -> Result<PathBuf> {
    if let Some(from_env) = first_existing_env(&locator.env_vars) {
        return Ok(from_env);
    }
    if let Ok(from_cdp) = default_executable() {
        return Ok(from_cdp);
    }
    if let Some(from_list) = locator.executables.iter().find(|path| path.is_file()) {
        return Ok(from_list.clone());
    }
    if let Some(from_home) = first_home_relative(&locator.home_relative) {
        return Ok(from_home);
    }
    Err(Error::msg(
        "Chromium/Chrome not found. Install Chrome or set one of the configured env vars to chrome.exe",
    ))
}

fn first_existing_env(keys: &[String]) -> Option<PathBuf> {
    for key in keys {
        if let Ok(value) = std::env::var(key) {
            let path = PathBuf::from(value);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn first_home_relative(relative_paths: &[PathBuf]) -> Option<PathBuf> {
    let home = user_home()?;
    for relative in relative_paths {
        let candidate = join_home(&home, relative);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn user_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn join_home(home: &Path, relative: &Path) -> PathBuf {
    let mut path = home.to_path_buf();
    for part in relative.iter() {
        path.push(part);
    }
    path
}
