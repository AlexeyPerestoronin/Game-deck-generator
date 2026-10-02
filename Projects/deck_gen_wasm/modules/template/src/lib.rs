//! GitHub new-game template, Deck-Games catalog, and bundled user help.

mod catalog;
mod github;
mod help;
mod template;

pub use crate::catalog::{
    catalog_game_roots, game_roots_from_paths, info_matches_query, install_catalog_game,
    load_catalog, parse_info_json5, CatalogEntry, GameInfo, DEFAULT_ICON_PNG, DEFAULT_PREVIEW_HTML,
};
pub use crate::github::list_game_folders;
pub use crate::help::{install_user_help, needs_install};
pub use crate::template::{install_game, install_new_game, InstalledGame};
