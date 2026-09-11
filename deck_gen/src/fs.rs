//! Filesystem used by load / catalog / HTML render.
//! Native CLI uses [`OsFs`]; the WASM app supplies a VFS adapter.

use std::path::{Path, PathBuf};

use crate::error::Result;

pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn write(&self, path: &Path, contents: &str) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn canonicalize(&self, path: &Path) -> Result<PathBuf>;
    fn search_roots(&self) -> Vec<PathBuf>;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct OsFs;

#[cfg(not(target_arch = "wasm32"))]
impl FileSystem for OsFs {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn write(&self, path: &Path, contents: &str) -> Result<()> {
        Ok(std::fs::write(path, contents)?)
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        Ok(std::fs::create_dir_all(path)?)
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(path)? {
            out.push(entry?.path());
        }
        Ok(out)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        Ok(crate::conf::canonicalize_or_abs(path))
    }

    fn search_roots(&self) -> Vec<PathBuf> {
        crate::conf::os_search_roots()
    }
}
