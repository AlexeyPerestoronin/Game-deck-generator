//! `${...}` expansion. Inside a JSON string the value is escaped string
//! content; outside it is a full JSON token.
//!
//! Resolution is a callback so load can keep file refs on pass 1 (`Keep`) and
//! substitute everything on pass 2 without a second scanner. [`lookup_var`]
//! walks dotted paths on a JSON tree (vars files or the synthetic `{vars: …}`).

use serde_json::Value;

use super::cursor::JsonText;
use crate::error::{Error, Result};

/// What to write in place of one `${…}` occurrence.
#[derive(Debug)]
pub enum Resolve {
    /// Leave the original `${…}` text unchanged (pass 1, local vars).
    Keep,
    /// Insert this JSON value, encoded for the current string context.
    Value(Value),
}

/// Walk `path` (`a.b.c`) on `tree`. Empty paths and missing keys are errors.
pub fn lookup_var<'a>(tree: &'a Value, path: &str) -> Result<&'a Value> {
    let mut node = tree;
    let parts: Vec<&str> = path.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return Err(Error::msg("Empty variable path in ${}"));
    }
    for part in &parts {
        match node {
            Value::Object(map) if map.contains_key(*part) => {
                node = &map[*part];
            }
            _ => {
                return Err(Error::msg(format!("Unknown variable ${{{}}}", path)));
            }
        }
    }
    Ok(node)
}

/// Replace each `${…}` in `raw` using `resolve`.
///
/// `resolve` is a generic `FnMut` (static dispatch). `${` is ignored when the
/// previous character in a string was `\`.
pub fn substitute_placeholders(
    raw: &str,
    mut resolve: impl FnMut(&str) -> Result<Resolve>,
) -> Result<String> {
    let mut scan = JsonText::new(raw);
    let mut out = String::new();
    while scan.remaining() {
        if !scan.escaped && scan.starts_with("${") {
            let (key, end) = read_placeholder(raw, scan.i)?;
            match resolve(key)? {
                Resolve::Keep => out.push_str(&raw[scan.i..end]),
                Resolve::Value(value) => {
                    out.push_str(&encode_placeholder(&value, scan.in_string)?);
                }
            }
            scan.skip_to(end);
            continue;
        }
        out.push(scan.take());
    }
    Ok(out)
}

fn read_placeholder(raw: &str, start: usize) -> Result<(&str, usize)> {
    if !raw[start..].starts_with("${") {
        return Err(Error::msg("Expected ${"));
    }
    let rel = raw[start + 2..]
        .find('}')
        .ok_or_else(|| Error::msg(format!("Unclosed placeholder at position {start}")))?;
    let end = start + 2 + rel;
    Ok((raw[start + 2..end].trim(), end + 1))
}

fn encode_placeholder(value: &Value, in_string: bool) -> Result<String> {
    if in_string {
        if let Value::String(text) = value {
            return escape_json_string_content(text);
        }
        return escape_json_string_content(&dumps(value)?);
    }
    dumps(value)
}

/// Compact JSON encoding of `value`.
pub fn dumps(value: &Value) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

/// `text` as JSON string content (no surrounding quotes).
pub fn escape_json_string_content(text: &str) -> Result<String> {
    let quoted = dumps(&Value::String(text.to_string()))?;
    Ok(quoted[1..quoted.len() - 1].to_string())
}
