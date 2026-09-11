//! Syntax highlighting via `syntect` (TextMate grammars).
//!
//! Used only for extensions the editor cares about (md, json/json5, html,
//! scss/css). Unknown types stay as a plain textarea. The syntax set is
//! loaded once; output is classed HTML for the overlay `<pre>`.

use std::sync::OnceLock;

use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::fs::file_ext;

static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();

fn syntax_set() -> &'static SyntaxSet {
    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn syntax_name_for_ext(ext: &str) -> Option<&'static str> {
    match ext.to_ascii_lowercase().as_str() {
        "md" | "markdown" => Some("md"),
        "json" | "json5" => Some("json"),
        "html" | "htm" => Some("html"),
        "scss" | "sass" | "css" => Some("css"),
        _ => None,
    }
}

/// Whether `path` has a grammar this module will highlight.
pub fn can_highlight(path: &str) -> bool {
    file_ext(path)
        .and_then(syntax_name_for_ext)
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlights_json_and_skips_pdf() {
        assert!(can_highlight("decks/data.json5"));
        assert!(can_highlight("readme.md"));
        assert!(!can_highlight("notes.txt"));
        assert!(!can_highlight("print.pdf"));
        let html = highlight_html("x.json", r#"{ "a": 1 }"#).expect("json highlight");
        assert!(html.contains("string") || html.contains("constant") || html.contains("span"));
    }
}

/// Classed HTML for `code`, or `None` if the extension is unknown / parse fails.
pub fn highlight_html(path: &str, code: &str) -> Option<String> {
    let ext = file_ext(path)?;
    let name = syntax_name_for_ext(ext)?;
    let set = syntax_set();
    let syntax = set.find_syntax_by_extension(name)?;
    let mut generator =
        ClassedHTMLGenerator::new_with_class_style(syntax, set, ClassStyle::Spaced);
    for line in LinesWithEndings::from(code) {
        generator.parse_html_for_line_which_includes_newline(line).ok()?;
    }
    Some(generator.finalize())
}
