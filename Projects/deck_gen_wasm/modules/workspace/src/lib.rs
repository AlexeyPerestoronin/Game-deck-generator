//! Reactive workspace state and editor actions.

mod actions;
mod ai;
mod commands;
mod copy_plan;
mod games;
mod split;
mod state;

pub use crate::ai::{ai_edit_target, list_ai_models};
pub use crate::copy_plan::row_looks_selected;
pub use crate::games::{game_card_name, game_menu_name, local_games, LocalGame};
pub use crate::split::is_previewable;
pub use crate::state::{OpenTab, TabKind, Workspace};
pub use deck_gen_wasm_ai::AiRequest;
