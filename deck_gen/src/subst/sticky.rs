//! `$s{...}` — after other placeholders, spaces become NBSP so a phrase
//! stays on one line in the card layout.

use super::cursor::JsonText;
use super::placeholder::escape_json_string_content;
use crate::error::{Error, Result};

const NBSP: char = '\u{00a0}';

pub fn expand_sticky_spans(raw: &str) -> Result<String> {
    let mut scan = JsonText::new(raw);
    let mut out = String::new();
    while scan.remaining() {
        if scan.in_string && !scan.escaped && scan.starts_with("$s{") {
            let (inner, end) = read_braced_inner(raw, scan.i + 2)?;
            let sticky = inner.replace(' ', &NBSP.to_string());
            out.push_str(&escape_json_string_content(&sticky)?);
            scan.skip_to(end);
            continue;
        }
        out.push(scan.take());
    }
    Ok(out)
}

fn read_braced_inner(raw: &str, start: usize) -> Result<(&str, usize)> {
    let bytes = raw.as_bytes();
    if start >= raw.len() || bytes[start] != b'{' {
        return Err(Error::msg("Expected {"));
    }
    let mut depth = 0usize;
    for (offset, ch) in raw[start..].char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                let end = start + offset;
                return Ok((&raw[start + 1..end], end + 1));
            }
        }
    }
    Err(Error::msg(format!("Unclosed $s{{ at position {start}")))
}
