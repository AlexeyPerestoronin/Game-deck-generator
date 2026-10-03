//! Overlay dialogs used by the activity bar and explorer.
//!
//! [`ConfirmModal`] is yes/no (clear workspace, load-game warning).
//! [`AlertModal`] is a single OK for prepare-html / import / load-files errors.
//! [`AiPromptModal`] is the one-shot AI prompt (create game / edit file).

mod ai_prompt;
mod alert;
mod confirm;

pub use ai_prompt::AiPromptModal;
pub use alert::AlertModal;
pub use confirm::ConfirmModal;
