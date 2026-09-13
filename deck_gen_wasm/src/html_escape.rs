//! HTML escaping used by the overlay editor and `srcdoc` iframe inlining.
//!
//! Two policies on purpose: visible text must escape `& < >`, while a `srcdoc`
//! attribute must escape `& " <` (not `>`) so the quoted attribute stays closed.

/// Escape `&`, `<`, and `>` for insertion into HTML text / overlay `<pre>`.
pub fn text_to_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Escape `&`, `"`, and `<` for a double-quoted `srcdoc="…"` attribute value.
pub fn srcdoc_attr(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    for ch in html.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_escapes_amp_lt_gt() {
        assert_eq!(text_to_html("a&b<c>d"), "a&amp;b&lt;c&gt;d");
    }

    #[test]
    fn srcdoc_escapes_amp_quot_lt_not_gt() {
        assert_eq!(srcdoc_attr(r#"a&b"c<d>e"#), "a&amp;b&quot;c&lt;d>e");
    }
}
