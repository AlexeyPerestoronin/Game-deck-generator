//! ZIP export of the workspace (encoding + browser download).

mod browser;
mod zip;

pub use browser::save_zip_bytes;
pub use zip::vfs_to_zip;

pub const ZIP_FILENAME: &str = "workspace.zip";
