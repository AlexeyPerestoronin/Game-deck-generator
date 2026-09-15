//! Public VFS types, path helpers, and file-kind predicates.

pub use crate::path::{
    file_ext, file_name, join_path, parent_path, path_is_or_under, retain_not_under, rewrite_prefix,
    rewrite_set, split_path, unique_name,
};
pub use crate::vfs::{Node, Vfs};
pub use crate::vfs_fs::VfsFs;

/// Extension classification used by import, preview, highlight, and icons.
pub mod kind {
    pub use crate::file_kind::*;
}
