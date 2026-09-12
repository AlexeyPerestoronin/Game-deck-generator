//! Which files may be imported from disk, and how rejects are explained.
//!
//! Text extensions live in [`crate::conf::import::ALLOWED_EXTENSIONS`]; image
//! types and the 100 MiB cap live next to them. The picker only asks
//! “is this path allowed / too large?” and, if not, shows
//! [`import_block_message`].

use crate::conf;
use crate::fs::file_ext;

/// Lowercased extension of `path`, if any.
fn ext_of(path: &str) -> Option<String> {
    file_ext(path).map(|ext| ext.to_ascii_lowercase())
}

fn ext_in(ext: &str, list: &[&str]) -> bool {
    list.iter().any(|ok| ext == *ok)
}

/// Whether `path` has an allowed text or image extension.
pub fn extension_allowed(path: &str) -> bool {
    match ext_of(path) {
        Some(ext) => {
            ext_in(&ext, conf::import::ALLOWED_EXTENSIONS)
                || ext_in(&ext, conf::import::IMAGE_EXTENSIONS)
        }
        None => false,
    }
}

/// Whether `path` is an accepted image type (jpg / png / icon and aliases).
pub fn is_image(path: &str) -> bool {
    match ext_of(path) {
        Some(ext) => ext_in(&ext, conf::import::IMAGE_EXTENSIONS),
        None => false,
    }
}

/// How one disk path should be imported, given its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportClass {
    /// Allowed text extension.
    Text,
    /// Allowed image within the size cap.
    Image,
    /// Extension is not on the allow-lists.
    Rejected,
    /// Image larger than [`conf::import::MAX_IMAGE_BYTES`].
    Oversized,
}

/// Classify `path` for import using `size_bytes` (from the File API).
pub fn classify(path: &str, size_bytes: u64) -> ImportClass {
    if !extension_allowed(path) {
        ImportClass::Rejected
    } else if is_image(path) && image_too_large(size_bytes) {
        ImportClass::Oversized
    } else if is_image(path) {
        ImportClass::Image
    } else {
        ImportClass::Text
    }
}

/// Whether an image of `size_bytes` exceeds [`conf::import::MAX_IMAGE_BYTES`].
pub fn image_too_large(size_bytes: u64) -> bool {
    size_bytes > conf::import::MAX_IMAGE_BYTES
}

/// `accept` value for `<input type="file">` (dot-prefixed, comma-separated).
pub fn file_input_accept() -> String {
    conf::import::ALLOWED_EXTENSIONS
        .iter()
        .chain(conf::import::IMAGE_EXTENSIONS.iter())
        .map(|ext| format!(".{ext}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Human-readable list of every allowed extension.
pub fn allowed_extension_list() -> String {
    conf::import::ALLOWED_EXTENSIONS
        .iter()
        .chain(conf::import::IMAGE_EXTENSIONS.iter())
        .copied()
        .collect::<Vec<_>>()
        .join(", ")
}

/// Combined dialog body for bad extensions and/or oversized images.
pub fn import_block_message(rejected: &[String], oversized: &[String]) -> String {
    let mut parts = Vec::new();
    if !oversized.is_empty() {
        parts.push(oversized_message(oversized));
    }
    if !rejected.is_empty() {
        parts.push(reject_message(rejected));
    }
    parts.join("\n\n")
}

/// Dialog body listing blocked relative paths (capped).
pub fn reject_message(rejected: &[String]) -> String {
    format!(
        "Cannot load because of files with extensions other than {}.\n\nBlocked files: {}",
        allowed_extension_list(),
        capped_list(rejected)
    )
}

/// Dialog body listing images over the size cap (capped).
pub fn oversized_message(oversized: &[String]) -> String {
    format!(
        "Cannot load because an image is larger than 100 MB.\n\nOversized files: {}",
        capped_list(oversized)
    )
}

fn capped_list(paths: &[String]) -> String {
    let limit = conf::import::REJECT_LIST_LIMIT;
    let shown: Vec<&str> = paths.iter().take(limit).map(String::as_str).collect();
    let extra = if paths.len() > limit {
        format!(" (and {} more)", paths.len() - limit)
    } else {
        String::new()
    };
    format!("{}{}", shown.join(", "), extra)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_listed_text_and_image_extensions() {
        assert!(extension_allowed("help.md"));
        assert!(extension_allowed("data.json5"));
        assert!(extension_allowed("views/simple-front.html"));
        assert!(extension_allowed("art/logo.png"));
        assert!(extension_allowed("art/photo.JPG"));
        assert!(extension_allowed("art/app.icon"));
        assert!(!extension_allowed("print.pdf"));
        assert!(!extension_allowed("notes.txt"));
        assert!(!extension_allowed("LICENSE"));
    }

    #[test]
    fn image_size_cap_is_100_mib() {
        assert!(!image_too_large(conf::import::MAX_IMAGE_BYTES));
        assert!(!image_too_large(0));
        assert!(image_too_large(conf::import::MAX_IMAGE_BYTES + 1));
        assert!(is_image("face.jpg"));
        assert!(is_image("mark.PNG"));
        assert!(!is_image("help.md"));
        assert_eq!(classify("help.md", 10), ImportClass::Text);
        assert_eq!(classify("logo.png", 10), ImportClass::Image);
        assert_eq!(
            classify("logo.png", conf::import::MAX_IMAGE_BYTES + 1),
            ImportClass::Oversized
        );
        assert_eq!(classify("notes.txt", 10), ImportClass::Rejected);
    }

    #[test]
    fn oversized_message_names_the_file() {
        let msg = oversized_message(&["art/huge.png".into()]);
        assert!(msg.contains("100 MB"), "{msg}");
        assert!(msg.contains("art/huge.png"), "{msg}");
    }
}
