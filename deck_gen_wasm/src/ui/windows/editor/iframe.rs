//! Inline VFS-relative `<iframe src>` into `srcdoc` for HTML preview.
//!
//! Nested `<iframe src="face.html">` inside `srcdoc` resolves against the IDE
//! page, so the app chrome is painted into the card strip. Sibling files from
//! the workspace tree are inlined instead.

use crate::fs::{join_path, parent_path, Vfs};

/// Rewrite relative iframe `src` attributes in `html` using VFS siblings of `html_path`.
pub fn inline_relative_iframes(vfs: &Vfs, html_path: &str, html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while let Some(rel) = lower[i..].find("<iframe") {
        let start = i + rel;
        out.push_str(&html[i..start]);
        let Some(gt) = html[start..].find('>') else {
            out.push_str(&html[start..]);
            return out;
        };
        let end = start + gt;
        out.push_str(&rewrite_iframe_tag(vfs, html_path, &html[start..end]));
        out.push('>');
        i = end + 1;
    }
    out.push_str(&html[i..]);
    out
}

fn rewrite_iframe_tag(vfs: &Vfs, html_path: &str, tag: &str) -> String {
    if attr_value(tag, "srcdoc").is_some() {
        return tag.to_string();
    }
    let Some((quote, url, span)) = attr_span(tag, "src") else {
        return tag.to_string();
    };
    let Some(resolved) = resolve_relative(html_path, url) else {
        return tag.to_string();
    };
    let Some(content) = vfs.read_file(&resolved) else {
        return tag.to_string();
    };
    let mut out = String::with_capacity(tag.len() + content.len());
    out.push_str(&tag[..span.start]);
    out.push_str("srcdoc=");
    out.push(quote);
    out.push_str(&escape_srcdoc(content));
    out.push(quote);
    out.push_str(&tag[span.end..]);
    out
}

fn attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    attr_span(tag, name).map(|(_, value, _)| value)
}

fn attr_span<'a>(tag: &'a str, name: &str) -> Option<(char, &'a str, std::ops::Range<usize>)> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{name}=");
    let mut from = 0;
    while let Some(rel) = lower[from..].find(&needle) {
        let start = from + rel;
        if start > 0 {
            let prev = tag.as_bytes()[start - 1];
            if !prev.is_ascii_whitespace() {
                from = start + 1;
                continue;
            }
        }
        let val_at = start + needle.len();
        let bytes = tag.as_bytes();
        let quote = *bytes.get(val_at)? as char;
        if quote != '"' && quote != '\'' {
            from = start + 1;
            continue;
        }
        let rest = &tag[val_at + 1..];
        let close = rest.find(quote)?;
        let end = val_at + 1 + close + 1;
        return Some((quote, &rest[..close], start..end));
    }
    None
}

fn resolve_relative(from_file: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || is_external_src(href) {
        return None;
    }
    let href = href.strip_prefix("./").unwrap_or(href);
    if href.starts_with('/')
        || href
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(join_path(&parent_path(from_file), href))
}

fn is_external_src(src: &str) -> bool {
    src.starts_with("//")
        || src.starts_with('#')
        || src.contains("://")
        || src.starts_with("data:")
        || src.starts_with("blob:")
        || src.starts_with("about:")
        || src.starts_with("javascript:")
}

fn escape_srcdoc(html: &str) -> String {
    html.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlines_sibling_iframe_src() {
        let mut vfs = Vfs::default();
        vfs.put_file("d/face.html", r#"<p class="c">card</p>"#.into())
            .unwrap();
        let preview = r#"<iframe class="faces" src="face.html" title="Face"></iframe>"#;
        let out = inline_relative_iframes(&vfs, "d/preview.html", preview);
        assert!(out.contains(r#"srcdoc="&lt;p class=&quot;c&quot;>card&lt;/p>""#));
        assert!(!out.contains(r#"src="face.html""#));
    }

    #[test]
    fn leaves_external_iframe_src() {
        let vfs = Vfs::default();
        let preview = r#"<iframe src="https://example.com"></iframe>"#;
        let out = inline_relative_iframes(&vfs, "d/preview.html", preview);
        assert_eq!(out, preview);
    }
}
