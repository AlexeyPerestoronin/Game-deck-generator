//! Deck model after JSON5 substitution.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::card::{card_from_row, normalize_map};
use crate::conf::output_dir_for_name;
use crate::error::{Error, Result};
use crate::load::DataManager;

pub struct Deck {
    pub name: String,
    pub directory: PathBuf,
    pub data_path: PathBuf,
    pub fields: Map<String, Value>,
    card_rows: Vec<Value>,
}

impl Deck {
    pub fn from_manager(manager: &DataManager) -> Result<Self> {
        let payload = manager.get_data()?;
        let (deck_fields, card_rows, extras) = split_payload(&payload, &manager.path)?;

        let mut data = normalize_map(&deck_fields);
        let extra_fields = normalize_map(&extras);
        for (key, value) in &extra_fields {
            data.insert(key.clone(), value.clone());
        }

        let name = extra_fields
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| Error::file(&manager.path, "must contain 'name'"))?
            .to_string();
        if !data.contains_key("view") {
            return Err(Error::file(&manager.path, "must contain 'view'"));
        }
        data.insert("key".into(), Value::String(name.clone()));

        Ok(Self {
            name,
            directory: manager.directory.clone(),
            data_path: manager.path.clone(),
            fields: data,
            card_rows,
        })
    }

    pub fn template_for(&self, side: &str, default: &str) -> String {
        self.fields
            .get("view")
            .and_then(Value::as_object)
            .and_then(|view| view.get(side))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| default.to_string())
    }

    pub fn output_dir(&self) -> Result<PathBuf> {
        output_dir_for_name(&self.name)
    }

    pub fn card_width_mm(&self) -> f64 {
        number(&self.fields["card_width_mm"])
    }

    pub fn card_height_mm(&self) -> f64 {
        number(&self.fields["card_height_mm"])
    }

    pub fn game_name(&self) -> String {
        self.fields
            .get("game_name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    }

    pub fn cards(&self) -> Vec<Value> {
        let defaults: Map<String, Value> = self
            .fields
            .iter()
            .filter(|(key, _)| key.starts_with("card_"))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        self.card_rows
            .iter()
            .enumerate()
            .map(|(index, row)| card_from_row(index + 1, row, &defaults))
            .collect()
    }
}

fn split_payload(
    payload: &Map<String, Value>,
    path: &Path,
) -> Result<(Map<String, Value>, Vec<Value>, Map<String, Value>)> {
    let deck_fields = payload
        .get("deck")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| Error::file(path, "must contain object 'deck'"))?;
    let card_rows = payload
        .get("cards")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| Error::file(path, "must contain list 'cards'"))?;
    let extras: Map<String, Value> = payload
        .iter()
        .filter(|(key, _)| *key != "deck" && *key != "cards")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    Ok((deck_fields, card_rows, extras))
}

fn number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(text) => text.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}
