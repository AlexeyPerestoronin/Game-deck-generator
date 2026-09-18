//! ZIP encode and browser download of the workspace.

mod browser;
mod zip;

pub use crate::browser::save_zip_bytes;
pub use crate::zip::vfs_to_zip;
pub use deck_gen_wasm_conf::export::ZIP_FILENAME;
