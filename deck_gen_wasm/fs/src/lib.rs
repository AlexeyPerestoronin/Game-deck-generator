//! In-memory VFS tree (`Node` + `Vfs`) and supporting path/kind utilities.
//!
//! This crate owns the workspace data model used by the browser editor.
//! - `Node` is the recursive element (file bytes or directory).
//! - `Vfs` is the root container with high-level operations (mkdir, read/write, copy...).
//! - Path helpers and `kind` classification are also provided.
//!
//! Binary/text policy and persistence strategy (localStorage vs IndexedDB) are
//! the responsibility of higher layers.

mod api;
mod file_kind;
mod path;
mod vfs;

pub use api::*;
