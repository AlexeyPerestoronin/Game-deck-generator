//! Load one deck JSON5 file and expand its placeholders.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::fs::FileSystem;
use crate::subst::{
    expand_sticky_spans, extract_json_object_for_key, lookup_var, substitute_placeholders, Resolve,
};

pub struct DataManager<'a> {
    fs: &'a dyn FileSystem,
    pub path: PathBuf,
    pub directory: PathBuf,
    game_vars_dir: PathBuf,
}

impl<'a> DataManager<'a> {
    pub fn new(
        fs: &'a dyn FileSystem,
        json_data_file: impl AsRef<Path>,
        game_vars_dir: PathBuf,
    ) -> Result<Self> {
        let path = fs.canonicalize(json_data_file.as_ref())?;
        if !fs.is_file(&path) {
            return Err(Error::file(&path, "Deck data file does not exist"));
        }
        let directory = path
            .parent()
            .ok_or_else(|| Error::file(&path, "has no parent directory"))?
            .to_path_buf();
        Ok(Self {
            fs,
            game_vars_dir,
            directory,
            path,
        })
    }

    pub fn get_data(&self) -> Result<Map<String, Value>> {
        let raw = self.fs.read_to_string(&self.path)?;
        let expanded = self.expand_placeholders(&raw)?;
        let payload = parse_json5(&self.path, &expanded)?;
        match payload {
            Value::Object(map) => Ok(map),
            _ => Err(Error::file(&self.path, "must be a JSON object")),
        }
    }

    fn expand_placeholders(&self, raw: &str) -> Result<String> {
        let mut file_cache: HashMap<String, Value> = HashMap::new();
        let pass1 = self.expand_file_refs(raw, &mut file_cache)?;
        let local_vars = extract_json_object_for_key(&pass1, "vars")?
            .unwrap_or_else(|| Value::Object(Map::new()));
        let pass2 = self.expand_all_refs(&pass1, &mut file_cache, &local_vars)?;
        expand_sticky_spans(&pass2)
    }

    fn expand_file_refs(
        &self,
        raw: &str,
        file_cache: &mut HashMap<String, Value>,
    ) -> Result<String> {
        substitute_placeholders(raw, |placeholder| {
            if !placeholder.contains(':') {
                return Ok(Resolve::Keep);
            }
            let (file_stem, path) = split_file_ref(placeholder)?;
            let tree = self.load_named_vars(file_cache, file_stem)?;
            Ok(Resolve::Value(lookup_var(&tree, path)?.clone()))
        })
    }

    fn expand_all_refs(
        &self,
        raw: &str,
        file_cache: &mut HashMap<String, Value>,
        local_vars: &Value,
    ) -> Result<String> {
        substitute_placeholders(raw, |placeholder| {
            if placeholder.contains(':') {
                let (file_stem, path) = split_file_ref(placeholder)?;
                let tree = self.load_named_vars(file_cache, file_stem)?;
                return Ok(Resolve::Value(lookup_var(&tree, path)?.clone()));
            }
            let mut root = Map::new();
            root.insert("vars".into(), local_vars.clone());
            Ok(Resolve::Value(
                lookup_var(&Value::Object(root), placeholder)?.clone(),
            ))
        })
    }

    fn load_named_vars(
        &self,
        cache: &mut HashMap<String, Value>,
        name: &str,
    ) -> Result<Value> {
        if let Some(existing) = cache.get(name) {
            return Ok(existing.clone());
        }
        let path = self.vars_file_path(name)?;
        let loaded = parse_json5(&path, &self.fs.read_to_string(&path)?)?;
        cache.insert(name.to_string(), loaded.clone());
        Ok(loaded)
    }

    fn vars_file_path(&self, name: &str) -> Result<PathBuf> {
        if !is_vars_stem(name) {
            return Err(Error::msg(format!("Invalid vars file name {name:?}")));
        }
        let path = self.game_vars_dir.join(format!("{name}.json5"));
        if self.fs.is_file(&path) {
            return Ok(path);
        }
        Err(Error::msg(format!(
            "Vars file '{name}' not found in: {}",
            path.display()
        )))
    }
}

fn split_file_ref(placeholder: &str) -> Result<(&str, &str)> {
    let (name, path) = placeholder.split_once(':').ok_or_else(|| {
        Error::msg(format!("Expected file:path in placeholder {placeholder:?}"))
    })?;
    Ok((name.trim(), path.trim()))
}

fn is_vars_stem(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && Path::new(name).file_name().and_then(|n| n.to_str()) == Some(name)
}

fn parse_json5(path: &Path, text: &str) -> Result<Value> {
    json5::from_str(text).map_err(|err| Error::file(path, format!("is not valid JSON5: {err}")))
}
