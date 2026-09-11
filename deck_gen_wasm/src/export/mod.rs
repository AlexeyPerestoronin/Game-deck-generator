//! ZIP export of the workspace (encoding + browser download).
//!
//! [`vfs_to_zip`] walks the in-memory tree into a deflated buffer.
//! [`save_zip_bytes`] offers that buffer through the File System Access picker
//! when present, otherwise a temporary `<a download>` click.

mod browser;
mod zip;

pub use browser::save_zip_bytes;
pub use zip::vfs_to_zip;

/// Default download name for a full workspace archive.
pub const ZIP_FILENAME: &str = "workspace.zip";
