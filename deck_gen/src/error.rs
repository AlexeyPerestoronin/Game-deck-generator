//! Crate-wide error type for I/O, JSON/JSON5, templates, and path-tagged messages.
//!
//! Callers in the CLI print `{err}` and exit. The WASM adapter maps the same
//! type to a UI string. There is no extra error hierarchy: one enum is enough
//! for the pipeline.

use std::path::PathBuf;

/// Result alias used throughout the HTML pipeline.
pub type Result<T> = std::result::Result<T, Error>;

/// Recoverable failure while loading conf, expanding data, or rendering HTML.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Free-form message (unknown deck, missing vars file, …).
    #[error("{0}")]
    Message(String),
    /// Failure tied to a specific file path.
    #[error("{path}: {message}")]
    File { path: PathBuf, message: String },
    /// Underlying I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// `serde_json` encode/decode (placeholder dumps, CLI `--json`).
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// JSON5 parse of conf, deck data, or vars.
    #[error(transparent)]
    Json5(#[from] json5::Error),
    /// MiniJinja load or render.
    #[error(transparent)]
    Template(#[from] minijinja::Error),
    /// PDF parse, impose, or image embedding (`lopdf`).
    #[error(transparent)]
    Pdf(#[from] lopdf::Error),
}

impl Error {
    /// Build a [`Error::Message`].
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    /// Build a [`Error::File`] with a display path.
    pub fn file(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::File {
            path: path.into(),
            message: message.into(),
        }
    }
}
