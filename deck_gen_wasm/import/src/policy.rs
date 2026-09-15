//! Which files may be imported from disk, and how rejects are explained.
//!
//! Text extensions live in [`crate::conf::import::ALLOWED_EXTENSIONS`]; image
//! types and the 100 MiB cap live next to them. The picker only asks
//! “is this path allowed / too large?” and, if not, shows
//! [`import_block_message`].

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::kind;

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
    if !kind::extension_allowed(path) {
        ImportClass::Rejected
    } else if kind::is_image(path) && image_too_large(size_bytes) {
        ImportClass::Oversized
    } else if kind::is_image(path) {
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
        assert!(kind::extension_allowed("help.md"));
        assert!(kind::extension_allowed("data.json5"));
        assert!(kind::extension_allowed("views/simple-front.html"));
        assert!(kind::extension_allowed("art/logo.png"));
        assert!(kind::extension_allowed("art/photo.JPG"));
        assert!(kind::extension_allowed("art/app.icon"));
        assert!(kind::extension_allowed("views/fit-header-name.js"));
        assert!(kind::extension_allowed("views/face-layout.j2"));
        assert!(!kind::extension_allowed("print.pdf"));
        assert!(!kind::extension_allowed("notes.txt"));
        assert!(!kind::extension_allowed("LICENSE"));
    }

    #[test]
    fn image_size_cap_is_100_mib() {
        assert!(!image_too_large(conf::import::MAX_IMAGE_BYTES));
        assert!(!image_too_large(0));
        assert!(image_too_large(conf::import::MAX_IMAGE_BYTES + 1));
        assert!(kind::is_image("face.jpg"));
        assert!(kind::is_image("mark.PNG"));
        assert!(!kind::is_image("help.md"));
        assert_eq!(classify("help.md", 10), ImportClass::Text);
        assert_eq!(classify("logo.png", 10), ImportClass::Image);
        assert_eq!(classify("views/fit-header-name.js", 123), ImportClass::Text);
        assert_eq!(classify("views/face-layout.j2", 200), ImportClass::Text);
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

    #[test]
    fn file_input_accept_and_allowed_list_include_js_for_direct_and_folder() {
        // used for <input accept> in direct multi-file picker
        let accept = file_input_accept();
        assert!(accept.contains(".js"), "accept must offer *.js: {accept}");
        assert!(accept.contains(".j2"), "accept must offer *.j2: {accept}");
        assert!(accept.contains(".json5"), "accept keeps prior: {accept}");

        // human list shown in error dialogs for both pickers
        let listed = allowed_extension_list();
        assert!(listed.contains("js"), "list must mention js: {listed}");
        assert!(listed.contains("j2"), "list must mention j2: {listed}");

        // both direct files and folder contents go through classify -> extension_allowed
        assert_eq!(classify("script.js", 42), ImportClass::Text);
        assert_eq!(classify("views/util.js", 100), ImportClass::Text);
        assert_eq!(classify("partial.j2", 50), ImportClass::Text);
    }
}
