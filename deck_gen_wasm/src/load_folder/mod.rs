//! Pick a local folder or files and copy them into the workspace.
//!
//! Text extensions in [`crate::conf::import::ALLOWED_EXTENSIONS`] and image
//! types in [`crate::conf::import::IMAGE_EXTENSIONS`] are accepted. Any other
//! extension, or an image over 100 MiB, rejects the whole import. The folder
//! picker uses `showDirectoryPicker` when present, otherwise a hidden
//! `<input webkitdirectory>`. File picks use a hidden `<input type="file">`.
//! A game folder is copied under `games/` with a unique name; “load file(s)”
//! writes into the folder that was right-clicked.

mod input;
mod install;
mod pick_files;
mod picker;
mod policy;
mod read;

pub use install::{install_files, install_folder};
pub use pick_files::pick_and_read_files;
pub use picker::pick_and_read_folder;
pub use policy::is_image;

/// UTF-8 source body or raw image bytes read from disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileBody {
    /// Text file (`md`, `json`, `html`, …).
    Text(String),
    /// Image (or other binary) bytes.
    Bytes(Vec<u8>),
}

/// Outcome of the directory picker, including extension/size rejects.
pub enum PickResult {
    /// User dismissed the picker.
    Cancelled,
    /// I/O failure, disallowed extensions, or oversized images.
    Rejected(String),
    /// Folder name plus relative files and directories.
    Ready {
        name: String,
        files: Vec<(String, FileBody)>,
        dirs: Vec<String>,
    },
}

/// Outcome of the multi-file picker used by folder “load file(s)”.
pub enum PickFilesResult {
    /// User dismissed the picker.
    Cancelled,
    /// I/O failure, disallowed extensions, or oversized images.
    Rejected(String),
    /// File names (no directories) plus bodies.
    Ready { files: Vec<(String, FileBody)> },
}
