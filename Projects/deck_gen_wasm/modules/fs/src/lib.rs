//! In-memory VFS tree (`Node` + `Vfs`) and supporting path/kind utilities.
//!
//! This crate owns the workspace data model used by the browser editor.
//! - `Node` is the recursive element (file bytes or directory).
//! - `Vfs` is the root container with high-level operations (mkdir, read/write, copy...).
//! - Path helpers and `kind` classification are also provided.
//!
//! Binary/text policy and persistence strategy (localStorage vs IndexedDB) are
//! the responsibility of higher layers.

mod file_kind;
mod path;
mod vfs;
mod vfs_fs;

pub use crate::path::{
    file_name, join_path, parent_path, path_is_or_under, retain_not_under, rewrite_prefix,
    rewrite_set, unique_name,
};
pub use crate::vfs::{Node, Vfs};
pub use crate::vfs_fs::VfsFs;

/// Extension classification used by import, preview, highlight, and icons.
pub mod kind {
    pub use crate::file_kind::*;
}
