//! Pick a local folder and copy it into the workspace.
//!
//! Only extensions listed in [`crate::conf::import`] are accepted; any other
//! file rejects the whole import. The picker uses `showDirectoryPicker` when
//! present, otherwise a hidden `<input webkitdirectory>`. The folder is copied
//! under `games/` with a unique name.

mod install;
mod picker;
mod policy;
mod read;

pub use install::install_folder;
pub use picker::pick_and_read_folder;

/// Outcome of the directory picker, including extension-policy rejects.
pub enum PickResult {
    /// User dismissed the picker.
    Cancelled,
    /// I/O failure or disallowed extensions.
    Rejected(String),
    /// Folder name plus relative files and directories.
    Ready {
        name: String,
        files: Vec<(String, String)>,
        dirs: Vec<String>,
    },
}
