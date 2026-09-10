use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("{path}: {message}")]
    File { path: PathBuf, message: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Json5(#[from] json5::Error),
    #[error(transparent)]
    Template(#[from] minijinja::Error),
}

impl Error {
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub fn file(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::File {
            path: path.into(),
            message: message.into(),
        }
    }
}
