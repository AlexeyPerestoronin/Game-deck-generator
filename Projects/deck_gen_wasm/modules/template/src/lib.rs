//! GitHub new-game template and bundled user help.

mod github;
mod help;
mod template;

pub use crate::github::list_game_folders;
pub use crate::help::{install_user_help, needs_install};
pub use crate::template::{install_game, install_new_game, InstalledGame};
