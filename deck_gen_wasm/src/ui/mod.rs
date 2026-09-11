//! Leptos widgets for the in-browser workspace editor.
//!
//! Controls are grouped by kind so each file owns one element:
//! bars, windows (panes), buttons, menus, modals, tooltips, and icons.
//! [`crate::app`] only mounts [`ActivityBar`], [`Explorer`], and [`Editor`].

mod bars;
mod buttons;
mod icons;
mod menus;
mod modals;
mod tooltips;
mod windows;

pub use bars::ActivityBar;
pub use windows::{Editor, Explorer};
