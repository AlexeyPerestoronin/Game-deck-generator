//! Filesystem used by load, catalog, and HTML render.
//!
//! The pipeline never calls `std::fs` directly. Everything goes through
//! [`FileSystem`] so the same code runs on the host disk ([`OsFs`]) and on the
//! in-memory tree supplied by `deck_gen_wasm`. Prefer a concrete `F: FileSystem`
//! (static dispatch). `dyn FileSystem` stays object-safe for the rare case
//! where the implementation is picked at runtime.

use std::path::{Path, PathBuf};

use crate::error::Result;

/// Read, write, and listing operations required to load games and emit HTML/PDF.
pub trait FileSystem: Send + Sync {
    /// Read an entire UTF-8 file.
    fn read_to_string(&self, path: &Path) -> Result<String>;
    /// Create or replace a UTF-8 file.
    fn write(&self, path: &Path, contents: &str) -> Result<()>;
    /// Create or replace a file with raw bytes (PDF and other binaries).
    fn write_bytes(&self, path: &Path, contents: &[u8]) -> Result<()>;
    /// Read an entire file as bytes.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>>;
    /// Create `path` and any missing parents.
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    /// Whether `path` is an existing file.
    fn is_file(&self, path: &Path) -> bool;
    /// Whether `path` is an existing directory.
    fn is_dir(&self, path: &Path) -> bool;
    /// Immediate children of `path` (files and directories).
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    /// Stable absolute (or VFS-normalized) form of `path`.
    fn canonicalize(&self, path: &Path) -> Result<PathBuf>;
    /// Directories walked upward when locating the repository-root `conf.json5`.
    fn search_roots(&self) -> Vec<PathBuf>;
}

/// Native disk implementation used by the CLI.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Default)]
pub struct OsFs;

#[cfg(not(target_arch = "wasm32"))]
impl FileSystem for OsFs {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn write(&self, path: &Path, contents: &str) -> Result<()> {
        Ok(std::fs::write(path, contents)?)
    }

    fn write_bytes(&self, path: &Path, contents: &[u8]) -> Result<()> {
        Ok(std::fs::write(path, contents)?)
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
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
        Ok(canonicalize_or_abs(path))
    }

    fn search_roots(&self) -> Vec<PathBuf> {
        os_search_roots()
    }
}

/// CWD and the directory that contains this executable — starting points for
/// walking up to a root `conf.json5`.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn os_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
        }
    }
    roots
}

/// `canonicalize`, falling back to an absolute join with CWD, then stripping
/// the Windows `\\?\` prefix so later path joins stay printable.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn canonicalize_or_abs(path: &Path) -> PathBuf {
    let raw = path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
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
