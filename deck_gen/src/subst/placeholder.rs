//! `${...}` expansion. Inside a JSON string the value is escaped string
//! content; outside it is a full JSON token.

use serde_json::Value;

use super::cursor::JsonText;
use crate::error::{Error, Result};

#[derive(Debug)]
pub enum Resolve {
    Keep,
    Value(Value),
}

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

pub fn dumps(value: &Value) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

pub fn escape_json_string_content(text: &str) -> Result<String> {
    let quoted = dumps(&Value::String(text.to_string()))?;
    Ok(quoted[1..quoted.len() - 1].to_string())
}
