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

/// UTF-8 source body or raw image bytes read from disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileBody {
    /// Text file (`md`, `json`, `html`, …).
    Text(String),
    /// Image (or other binary) bytes.
    Bytes(Vec<u8>),
}

/// Outcome of a directory or file picker, including extension/size rejects.
pub enum PickOutcome<T> {
    /// User dismissed the picker.
    Cancelled,
    /// I/O failure, disallowed extensions, or oversized images.
    Rejected(String),
    /// Successful payload (`PickedFolder` or `PickedFiles`).
    Ready(T),
}

/// Folder name plus relative files and directories from a directory pick.
pub struct PickedFolder {
    /// Chosen folder name (unique-name suffix applied later at install).
    pub name: String,
    /// Relative files and bodies.
    pub files: Vec<(String, FileBody)>,
    /// Relative directory paths.
    pub dirs: Vec<String>,
}

/// File names (no directories) plus bodies from a multi-file pick.
pub struct PickedFiles {
    /// File names and bodies.
    pub files: Vec<(String, FileBody)>,
}

/// Outcome of the directory picker.
pub type PickResult = PickOutcome<PickedFolder>;

/// Outcome of the multi-file picker used by folder “load file(s)”.
pub type PickFilesResult = PickOutcome<PickedFiles>;
