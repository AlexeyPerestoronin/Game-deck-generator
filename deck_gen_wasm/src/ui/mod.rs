//! Leptos panes for the in-browser workspace editor.
//!
//! The shell is three siblings: [`ActivityBar`] (actions), [`Explorer`] (tree),
//! and [`Editor`] (open file). Supporting widgets (context menu, modals,
//! tooltips, icons, syntect highlighting) stay in this module tree so `app`
//! only wires layout.

mod activity_bar;
mod context_menu;
mod editor;
mod explorer;
mod highlight;
mod icons;
mod modal;
mod tooltip;

pub use activity_bar::ActivityBar;
pub use editor::Editor;
pub use explorer::Explorer;
