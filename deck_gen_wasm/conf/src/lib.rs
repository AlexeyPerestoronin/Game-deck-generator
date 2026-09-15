//! Compile-time knobs for the browser editor.
//!
//! Central place for constants that control GitHub template install,
//! session storage keys, import allow-lists (text + images), I/O
//! parallelism, and UI timing. Values are never mutated at runtime.
//! Both direct file loads and folder loads consult the lists here
//! (via ALLOWED_EXTENSIONS) so *.js (and future types) are enabled
//! in one place.

mod api;

pub use api::*;
