//! Clickable controls: activity-bar actions and explorer create buttons.
//!
//! Each button is a Leptos component in its own file so the bars and windows
//! compose them without embedding markup or click handlers.

mod clear;
mod download;
mod load_game;
mod new_file;
mod new_folder;
mod new_game;
mod prepare_html;
mod prepare_pdf;
mod save;

pub use clear::ClearButton;
pub use download::DownloadButton;
pub use load_game::LoadGameButton;
pub use new_file::NewFileButton;
pub use new_folder::NewFolderButton;
pub use new_game::NewGameButton;
pub use prepare_html::PrepareHtmlButton;
pub use prepare_pdf::PreparePdfButton;
pub use save::SaveButton;
