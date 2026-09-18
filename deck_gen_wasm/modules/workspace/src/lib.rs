//! Reactive workspace state and editor actions.

mod actions;
mod commands;
mod copy_plan;
mod split;
mod state;

pub use crate::copy_plan::row_looks_selected;
pub use crate::split::is_previewable;
pub use crate::state::{OpenTab, TabKind, Workspace};
