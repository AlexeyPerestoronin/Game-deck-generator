//! Pick local files or a folder and install them into the VFS.
//!
//! Provides the two entry points for user-driven import:
//! - pick_and_read_folder: directory picker (showDirectoryPicker or webkitdirectory fallback)
//! - pick_and_read_files: multi-file picker
//!
//! Both support the same allow-list from conf (text files incl. js + images).
//! Walkers classify each entry (reject bad ext / oversized), read bodies
//! (text or bytes) in parallel, and return Picked* ready for install.
//! Install copies into VFS under games/ (with unique name for folders).
//! All heavy lifting is here; UI only calls the pick_* fns.

mod input;
mod install;
mod pick_files;
mod picker;
mod policy;
mod read;
mod types;

pub use crate::install::{install_files, install_folder};
pub use crate::pick_files::pick_and_read_files;
pub use crate::picker::pick_and_read_folder;
pub use crate::types::{
    FileBody, PickFilesResult, PickOutcome, PickResult, PickedFiles, PickedFolder,
};
