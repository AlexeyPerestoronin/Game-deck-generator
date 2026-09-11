//! Slice a `"key": { ... }` object out of still-invalid JSON text.
//!
//! Local `vars` must be read after file-ref expansion and before the rest of
//! `${…}` is resolved, so the file is not valid JSON5 yet. This walks with
//! [`JsonText`](super::cursor::JsonText), finds `"key"`, and parses the following
//! braced object with `json5`.

use serde_json::Value;

use super::cursor::{skip_json_whitespace, JsonText};
use crate::error::{Error, Result};

/// First `"key": { … }` object in `raw`, or `None` if the key is absent.
pub fn extract_json_object_for_key(raw: &str, key: &str) -> Result<Option<Value>> {
    let needle = format!("\"{key}\"");
    let mut scan = JsonText::new(raw);
    while scan.remaining() {
        if !scan.in_string && scan.starts_with(&needle) {
            if let Some(value) = try_parse_object_after_key(raw, scan.i + needle.len())? {
                return Ok(Some(value));
            }
        }
        scan.take();
    }
    Ok(None)
}

fn try_parse_object_after_key(raw: &str, after_key: usize) -> Result<Option<Value>> {
    let mut index = skip_json_whitespace(raw, after_key);
    if index >= raw.len() || raw.as_bytes()[index] != b':' {
        return Ok(None);
    }
    index = skip_json_whitespace(raw, index + 1);
    if index >= raw.len() || raw.as_bytes()[index] != b'{' {
        return Ok(None);
    }
    let blob = slice_braced_object(raw, index)?;
    Ok(Some(json5::from_str(blob)?))
}

fn slice_braced_object(raw: &str, start: usize) -> Result<&str> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in raw[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok(&raw[start..start + offset + 1]);
            }
        }
    }
    Err(Error::msg("Unclosed JSON object"))
}
