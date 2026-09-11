//! One card row: key normalisation, life-effects, slug.
//!
//! Deck JSON5 is schema-light (`serde_json::Value`). This module turns a raw
//! row plus `card_*` defaults into the object the templates see: lowercase
//! keys, 1-based `index`, a `slug`, and `life_effects` split into a string
//! array. Regexes are compiled once; the splitting rules are part of the
//! published card layout.

use std::sync::OnceLock;

use regex::Regex;
use serde_json::{Map, Value};

fn normalize_key(key: &str) -> String {
    key.trim().replace('-', "_").to_lowercase()
}

/// Lowercase keys, map `-` to `_`, keep values. Used for both deck fields and
/// extra top-level keys from `data.json5`.
pub fn normalize_map(map: &Map<String, Value>) -> Map<String, Value> {
    map.iter()
        .map(|(key, value)| (normalize_key(key), value.clone()))
        .collect()
}

/// Build the JSON object for card `index` (1-based) from a row and defaults.
///
/// Defaults are the deck's `card_*` fields. Row keys override them. Empty
/// keys are ignored. `life_effects` is always an array of trimmed strings.
pub fn card_from_row(index: usize, row: &Value, defaults: &Map<String, Value>) -> Value {
    let mut fields = defaults.clone();
    if let Value::Object(map) = row {
        for (key, value) in map {
            if key.is_empty() {
                continue;
            }
            fields.insert(normalize_key(key), value.clone());
        }
    }
    fields.insert("index".into(), Value::from(index as u64));
    if let Some(raw) = fields.get("life_effects").cloned() {
        fields.insert("life_effects".into(), Value::Array(split_effects(&raw)));
    }
    fields.insert("slug".into(), Value::String(slug(&fields)));
    Value::Object(fields)
}

fn split_effects(raw: &Value) -> Vec<Value> {
    match raw {
        Value::Null => Vec::new(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                let trimmed = value_as_text(item).trim().to_string();
                (!trimmed.is_empty()).then_some(Value::String(trimmed))
            })
            .collect(),
        other => split_effects_text(&value_as_text(other)),
    }
}

fn split_effects_text(text: &str) -> Vec<Value> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    effects_split_regex()
        .split(trimmed)
        .filter_map(|chunk| {
            let line = effects_bullet_regex().replace(chunk, "");
            let line = line.trim();
            (!line.is_empty()).then_some(Value::String(line.to_string()))
        })
        .collect()
}

fn slug(fields: &Map<String, Value>) -> String {
    let name = fields
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or("card");
    let safe = slug_unsafe_regex().replace_all(name.trim(), "-");
    let safe = safe.trim_matches('-');
    let safe = if safe.is_empty() { "card" } else { safe };
    let index = fields.get("index").and_then(Value::as_u64).unwrap_or(0);
    format!("{index:03}-{safe}")
}

fn value_as_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn effects_split_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:/n|\\13\\|\n|\|)").expect("effects split pattern"))
}

fn effects_bullet_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[\s*•\-–—·]+").expect("effects bullet pattern"))
}

fn slug_unsafe_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[^\w\-]+").expect("slug pattern"))
}
