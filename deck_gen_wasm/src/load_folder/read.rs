//! Walk a directory handle or `FileList` into relative paths and UTF-8 bodies.
//!
//! The File System Access tree is an async iterator of `[name, handle]` pairs.
//! The `<input webkitdirectory>` path is a flat list with `webkitRelativePath`.
//! Both produce the same `(files, dirs, rejected)` triple for the installer.

use std::collections::BTreeSet;

use js_sys::{Array, Function, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileList};

use super::policy::extension_allowed;
use crate::conf;
use crate::fs::parent_path;

pub(super) fn split_webkit_path(path: &str) -> (Option<String>, String) {
    let path = path.replace('\\', "/");
    match path.split_once('/') {
        Some((root, rest)) if !root.is_empty() && !rest.is_empty() => {
            (Some(root.to_string()), rest.to_string())
        }
        _ => (None, path),
    }
}

pub(super) async fn collect_from_file_list(
    list: FileList,
) -> Result<(String, Vec<(String, String)>, Vec<String>, Vec<String>), String> {
    let mut folder_name = conf::import::DEFAULT_FOLDER_NAME.to_string();
    let mut files = Vec::new();
    let mut dirs = BTreeSet::new();
    let mut rejected = Vec::new();
    for index in 0..list.length() {
        let Some(file) = list.item(index) else {
            continue;
        };
        let relative = webkit_relative_path(&file);
        let (root, rel) = split_webkit_path(&relative);
        if let Some(root) = root {
            folder_name = root;
        }
        let mut parent = parent_path(&rel);
        while !parent.is_empty() {
            dirs.insert(parent.clone());
            parent = parent_path(&parent);
        }
        if extension_allowed(&rel) {
            files.push((rel, read_file_text(&file).await?));
        } else {
            rejected.push(rel);
        }
    }
    Ok((folder_name, files, dirs.into_iter().collect(), rejected))
}

fn webkit_relative_path(file: &File) -> String {
    Reflect::get(file, &JsValue::from_str("webkitRelativePath"))
        .ok()
        .and_then(|value| value.as_string())
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| file.name())
}

async fn read_file_text(file: &File) -> Result<String, String> {
    let value = JsFuture::from(file.text())
        .await
        .map_err(|_| "Could not read file".to_string())?;
    value
        .as_string()
        .ok_or_else(|| "Could not read file as text".to_string())
}

pub(super) async fn collect_tree(
    dir: &JsValue,
    prefix: &str,
) -> Result<(Vec<(String, String)>, Vec<String>, Vec<String>), String> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let mut rejected = Vec::new();
    let entries = call0(dir, "entries")?;
    loop {
        let next = call_async(&entries, "next").await?;
        let done = Reflect::get(&next, &JsValue::from_str("done"))
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(true);
        if done {
            break;
        }
        let value = Reflect::get(&next, &JsValue::from_str("value"))
            .map_err(|_| "folder entry is missing".to_string())?;
        let pair = value
            .dyn_into::<Array>()
            .map_err(|_| "folder entry is not a pair".to_string())?;
        let child_name = pair
            .get(0)
            .as_string()
            .ok_or_else(|| "folder entry has no name".to_string())?;
        if child_name == "." || child_name == ".." {
            continue;
        }
        let child = pair.get(1);
        let kind = Reflect::get(&child, &JsValue::from_str("kind"))
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_default();
        let rel = if prefix.is_empty() {
            child_name.clone()
        } else {
            format!("{prefix}/{child_name}")
        };
        if kind == "directory" {
            dirs.push(rel.clone());
            let (nested_files, nested_dirs, nested_rejected) =
                Box::pin(collect_tree(&child, &rel)).await?;
            files.extend(nested_files);
            dirs.extend(nested_dirs);
            rejected.extend(nested_rejected);
        } else if extension_allowed(&rel) {
            let file = call_async(&child, "getFile").await?;
            let text = call_async(&file, "text").await?;
            let content = text
                .as_string()
                .ok_or_else(|| format!("Could not read {rel}"))?;
            files.push((rel, content));
        } else {
            rejected.push(rel);
        }
    }
    Ok((files, dirs, rejected))
}

fn call0(receiver: &JsValue, method: &str) -> Result<JsValue, String> {
    let func = Reflect::get(receiver, &JsValue::from_str(method))
        .map_err(|_| format!("missing {method}"))?
        .dyn_into::<Function>()
        .map_err(|_| format!("{method} is not a function"))?;
    func.call0(receiver)
        .map_err(|_| format!("{method} failed"))
}

async fn call_async(receiver: &JsValue, method: &str) -> Result<JsValue, String> {
    let func = Reflect::get(receiver, &JsValue::from_str(method))
        .map_err(|_| format!("missing {method}"))?
        .dyn_into::<Function>()
        .map_err(|_| format!("{method} is not a function"))?;
    let promise = func
        .call0(receiver)
        .map_err(|_| format!("{method} failed"))?;
    JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|_| format!("{method} failed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_webkit_relative_path() {
        assert_eq!(
            split_webkit_path("demo/decks/data.json5"),
            (Some("demo".into()), "decks/data.json5".into())
        );
        assert_eq!(split_webkit_path("help.md"), (None, "help.md".into()));
    }
}
