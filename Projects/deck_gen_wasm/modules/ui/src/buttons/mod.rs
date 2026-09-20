//! Clickable controls: activity-bar actions (including split preview) and explorer create buttons.
//!
//! Each button is a Leptos component in its own file so the bars and windows
//! compose them without embedding markup or click handlers.

mod activity;
mod clear;
mod download;
mod explorer;
mod feedback;
mod games;
mod load_game;
mod locale;
mod new_file;
mod new_folder;
mod new_game;
mod prepare_html;
mod prepare_pdf;
mod settings;
mod split_preview;
mod theme;

pub use activity::ActivityButton;
pub use clear::ClearButton;
pub use download::DownloadButton;
pub use explorer::ExplorerButton;
pub use games::GamesButton;
pub use new_file::NewFileButton;
pub use new_folder::NewFolderButton;
pub use prepare_html::PrepareHtmlButton;
pub use prepare_pdf::PreparePdfButton;
pub use settings::SettingsButton;
pub use split_preview::SplitPreviewButton;
