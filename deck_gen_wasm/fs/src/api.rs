//! Public VFS types, path helpers, and file-kind predicates.
//!
//! The primary filesystem element is [`Node`]. [`Vfs`] provides convenient
//! high-level operations over a tree of nodes. Path utilities and `kind` are
//! supporting modules.

pub use crate::path::{
    file_name, join_path, parent_path, path_is_or_under, retain_not_under, rewrite_prefix,
    rewrite_set, unique_name,
};
pub use crate::vfs::{Node, Vfs};

/// Extension classification used by import, preview, highlight, and icons.
pub mod kind {
    pub use crate::file_kind::*;
}
