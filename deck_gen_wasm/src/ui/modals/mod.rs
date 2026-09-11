//! Overlay dialogs used by the activity bar.
//!
//! [`ConfirmModal`] is yes/no (clear workspace, load-game warning).
//! [`AlertModal`] is a single OK for prepare-html / import errors.

mod alert;
mod confirm;

pub use alert::AlertModal;
pub use confirm::ConfirmModal;
