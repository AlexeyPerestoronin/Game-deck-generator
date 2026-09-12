//! Which files may be imported from disk, and how rejects are explained.
//!
//! The allowed-extension list lives in [`crate::conf::import`]. The rest of
//! the picker only asks “is this path allowed?” and, if not, shows
//! [`reject_message`].

use crate::conf;
use crate::fs::file_ext;

/// Whether `path` has an allowed source-file extension.
pub fn extension_allowed(path: &str) -> bool {
    match file_ext(path).map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) => conf::import::ALLOWED_EXTENSIONS.iter().any(|ok| ext == *ok),
        None => false,
    }
}

/// Dialog body listing blocked relative paths (capped).
pub fn reject_message(rejected: &[String]) -> String {
    let limit = conf::import::REJECT_LIST_LIMIT;
    let shown: Vec<&str> = rejected.iter().take(limit).map(String::as_str).collect();
    let extra = if rejected.len() > limit {
        format!(" (and {} more)", rejected.len() - limit)
    } else {
        String::new()
    };
    format!(
        "This folder cannot be loaded because it contains files with extensions other than md, json, json5, html, scss.\n\nBlocked files: {}{}",
        shown.join(", "),
        extra
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_only_listed_extensions() {
        assert!(extension_allowed("help.md"));
        assert!(extension_allowed("data.json5"));
        assert!(extension_allowed("views/simple-front.html"));
        assert!(!extension_allowed("print.pdf"));
        assert!(!extension_allowed("notes.txt"));
        assert!(!extension_allowed("LICENSE"));
    }
}
