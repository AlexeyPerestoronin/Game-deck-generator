//! Floating menus (explorer context menu).
//!
//! File rows expose `copy`; folder rows expose `copy` and `past`. A copy row
//! is tinted blue while that path is in the workspace copy plan.

mod context;

pub use context::{ChosenCommand, ContextMenu, EntryKind, MenuState};
