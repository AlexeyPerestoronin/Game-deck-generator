//! Classify a path by extension: import, preview, highlight, icons, MIME.
//!
//! This is the single place that knows which extensions mean what for the UI
//! and import pipeline. The VFS itself treats all files uniformly as bytes.

use crate::path::file_ext;
use deck_gen_wasm_conf as conf;

/// High-level kind used by preview, highlight, and MIME.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    /// Markdown (`md`, `markdown`).
    Markdown,
    /// JSON.
    Json,
    /// JSON5.
    Json5,
    /// HTML (`html`, `htm`).
    Html,
    /// SCSS.
    Scss,
    /// `sass` / `css` (highlight only).
    Css,
    /// PDF.
    Pdf,
    /// jpg / png / ico / icon.
    Image,
    /// Anything else, including no extension.
    Other,
}

fn ext_in(ext: &str, list: &[&str]) -> bool {
    list.iter().copied().any(|ok| ext == ok)
}

fn lower_ext(path: &str) -> Option<String> {
    file_ext(path).map(str::to_ascii_lowercase)
}

/// Kind for `path`, or [`FileKind::Other`].
pub fn kind_of(path: &str) -> FileKind {
    let Some(ext) = lower_ext(path) else {
        return FileKind::Other;
    };
    let ext = ext.as_str();
    if ext_in(ext, conf::ext::MARKDOWN) {
        FileKind::Markdown
    } else if ext_in(ext, conf::ext::JSON) {
        FileKind::Json
    } else if ext_in(ext, conf::ext::JSON5) {
        FileKind::Json5
    } else if ext_in(ext, conf::ext::HTML) {
        FileKind::Html
    } else if ext_in(ext, conf::ext::SCSS) {
        FileKind::Scss
    } else if ext_in(ext, conf::ext::SASS_CSS) {
        FileKind::Css
    } else if ext_in(ext, conf::ext::PDF) {
        FileKind::Pdf
    } else if ext_in(ext, conf::ext::IMAGE) {
        FileKind::Image
    } else {
        FileKind::Other
    }
}

/// Whether `path` is an accepted image type (jpg / png / icon and aliases).
pub fn is_image(path: &str) -> bool {
    kind_of(path) == FileKind::Image
}

/// HTML, Markdown, PDF, and images can render in a preview tab.
pub fn is_previewable(path: &str) -> bool {
    matches!(
        kind_of(path),
        FileKind::Markdown | FileKind::Html | FileKind::Pdf | FileKind::Image
    )
}

/// Whether `path` has a syntect grammar this editor will highlight.
pub fn can_highlight(path: &str) -> bool {
    syntax_name(path).is_some()
}

/// syntect extension name for `path`, if any.
///
/// Supports `js` and `j2` directly (in addition to kinds that map to highlightable
/// grammar names). Returns the name passed to syntect's `find_syntax_by_extension`.
pub fn syntax_name(path: &str) -> Option<&'static str> {
    match lower_ext(path).as_deref() {
        Some("js") => Some("js"),
        Some("j2") => Some("j2"),
        _ => match kind_of(path) {
            FileKind::Markdown => Some("md"),
            FileKind::Json | FileKind::Json5 => Some("json"),
            FileKind::Html => Some("html"),
            FileKind::Scss | FileKind::Css => Some("css"),
            _ => None,
        },
    }
}

/// MIME type for an image (or octet-stream if the ext is unknown).
pub fn image_mime(path: &str) -> &'static str {
    match lower_ext(path).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("ico" | "icon") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

/// Whether `path` has an allowed text or image extension for disk import.
pub fn extension_allowed(path: &str) -> bool {
    match lower_ext(path) {
        Some(ext) => {
            ext_in(&ext, conf::import::ALLOWED_EXTENSIONS)
                || ext_in(&ext, conf::import::IMAGE_EXTENSIONS)
        }
        None => false,
    }
}

/// Explorer glyph kind. `htm` / `markdown` stay as empty slot (previous UI).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplorerIcon {
    /// Markdown (`md` only).
    Md,
    /// JSON.
    Json,
    /// JSON5.
    Json5,
    /// HTML (`html` only).
    Html,
    /// SCSS.
    Scss,
    /// JavaScript (`js` only).
    Js,
    /// Jinja2 (`j2` only).
    J2,
    /// PDF.
    Pdf,
    /// Image.
    Image,
}

/// Which explorer glyph `name` should use (`None` = empty slot).
pub fn explorer_icon(name: &str) -> Option<ExplorerIcon> {
    match lower_ext(name)?.as_str() {
        "md" => Some(ExplorerIcon::Md),
        "json" => Some(ExplorerIcon::Json),
        "json5" => Some(ExplorerIcon::Json5),
        "html" => Some(ExplorerIcon::Html),
        "scss" => Some(ExplorerIcon::Scss),
        "js" => Some(ExplorerIcon::Js),
        "j2" => Some(ExplorerIcon::J2),
        "pdf" => Some(ExplorerIcon::Pdf),
        ext if ext_in(ext, conf::ext::IMAGE) => Some(ExplorerIcon::Image),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previewable_html_md_pdf_images_only() {
        assert!(is_previewable("a/preview.html"));
        assert!(is_previewable("help.MD"));
        assert!(is_previewable("doc.markdown"));
        assert!(is_previewable("a/face.pdf"));
        assert!(is_previewable("a/logo.png"));
        assert!(is_previewable("a/mark.JPG"));
        assert!(is_previewable("a/app.icon"));
        assert!(!is_previewable("a/data.json5"));
        assert!(!is_previewable("a/style.scss"));
    }

    #[test]
    fn import_allows_listed_text_and_image() {
        assert!(extension_allowed("help.md"));
        assert!(extension_allowed("data.json5"));
        assert!(extension_allowed("views/simple-front.html"));
        assert!(extension_allowed("art/logo.png"));
        assert!(extension_allowed("art/photo.JPG"));
        assert!(extension_allowed("art/app.icon"));
        assert!(extension_allowed("views/fit-header-name.js"));
        assert!(extension_allowed("views/face-layout.j2"));
        assert!(!extension_allowed("print.pdf"));
        assert!(!extension_allowed("notes.txt"));
        assert!(!extension_allowed("LICENSE"));
        assert!(!extension_allowed("doc.markdown"));
        assert!(!extension_allowed("page.htm"));
    }

    #[test]
    fn explorer_icon_keeps_htm_markdown_empty() {
        assert_eq!(explorer_icon("help.md"), Some(ExplorerIcon::Md));
        assert_eq!(explorer_icon("doc.markdown"), None);
        assert_eq!(explorer_icon("page.html"), Some(ExplorerIcon::Html));
        assert_eq!(explorer_icon("page.htm"), None);
        assert_eq!(explorer_icon("logo.png"), Some(ExplorerIcon::Image));
        assert_eq!(explorer_icon("face.pdf"), Some(ExplorerIcon::Pdf));
        assert_eq!(explorer_icon("notes.txt"), None);
        assert_eq!(explorer_icon("a.js"), Some(ExplorerIcon::Js));
        assert_eq!(explorer_icon("a.j2"), Some(ExplorerIcon::J2));
        assert_eq!(explorer_icon("SCRIPT.JS"), Some(ExplorerIcon::Js));
    }

    #[test]
    fn highlight_includes_markdown_htm_css() {
        assert!(can_highlight("readme.md"));
        assert!(can_highlight("readme.markdown"));
        assert!(can_highlight("decks/data.json5"));
        assert!(can_highlight("a.htm"));
        assert!(can_highlight("a.css"));
        assert!(can_highlight("script.js"));
        assert!(can_highlight("partial.j2"));
        assert!(!can_highlight("notes.txt"));
        assert!(!can_highlight("print.pdf"));
    }
}
