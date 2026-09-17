//! Main IDE panes: file tree and the open-file editor (optionally split).

mod editor;
mod explorer;
mod games;

pub use editor::Editor;
pub use explorer::Explorer;
pub use games::GamesPanel;
