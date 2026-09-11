//! Pick a local folder and copy it into the workspace.

use js_sys::{Array, Function, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::fs::{file_ext, join_path, Vfs};

const ALLOWED: &[&str] = &["md", "json", "json5", "html", "scss"];

pub enum PickResult {
    Cancelled,
    Rejected(String),
    Ready { name: String, files: Vec<(String, String)>, dirs: Vec<String> },
}

pub fn extension_allowed(path: &str) -> bool {
    match file_ext(path).map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) => ALLOWED.iter().any(|ok| ext == *ok),
        None => false,
    }
}

pub fn unique_folder_name(base: &str, taken: impl Fn(&str) -> bool) -> String {
    let base = if base.is_empty() { "game" } else { base };
    if !taken(base) {
        return base.to_string();
    }
    let mut n = 1u32;
    loop {
        let name = format!("{base}-{n}");
        if !taken(&name) {
            return name;
        }
        n += 1;
    }
}

pub fn install_folder(
    vfs: &mut Vfs,
    name: &str,
    dirs: &[String],
    files: &[(String, String)],
) -> Result<String, String> {
    let folder = unique_folder_name(name, |candidate| vfs.exists(&format!("games/{candidate}")));
    let root = format!("games/{folder}");
    vfs.mkdir(&root)?;
    for dir in dirs {
        vfs.mkdir(&join_path(&root, dir))?;
    }
    for (rel, content) in files {
        vfs.put_file(&join_path(&root, rel), content.clone())?;
    }
    Ok(folder)
}

pub async fn pick_and_read_folder() -> PickResult {
    let handle = match pick_directory().await {
        PickDir::Unsupported => {
            return PickResult::Rejected(
                "This browser cannot open a folder picker. Use Chrome or Edge.".into(),
            );
        }
        PickDir::Cancelled => return PickResult::Cancelled,
        PickDir::Handle(handle) => handle,
    };
    let name = Reflect::get(&handle, &JsValue::from_str("name"))
        .ok()
        .and_then(|value| value.as_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "game".to_string());
    match collect_tree(&handle, "").await {
        Ok((files, dirs, rejected)) => {
            if !rejected.is_empty() {
                PickResult::Rejected(reject_message(&rejected))
            } else {
                PickResult::Ready { name, files, dirs }
            }
        }
        Err(err) => PickResult::Rejected(err),
    }
}

fn reject_message(rejected: &[String]) -> String {
    let shown: Vec<&str> = rejected.iter().take(8).map(String::as_str).collect();
    let extra = if rejected.len() > 8 {
        format!(" (and {} more)", rejected.len() - 8)
    } else {
        String::new()
    };
    format!(
        "This folder cannot be loaded because it contains files with extensions other than md, json, json5, html, scss.\n\nBlocked files: {}{}",
        shown.join(", "),
        extra
    )
}

enum PickDir {
    Unsupported,
    Cancelled,
    Handle(JsValue),
}

async fn pick_directory() -> PickDir {
    let Some(window) = web_sys::window() else {
        return PickDir::Unsupported;
    };
    let has_picker = Reflect::has(&window, &JsValue::from_str("showDirectoryPicker")).unwrap_or(false);
    if !has_picker {
        return PickDir::Unsupported;
    }
    let Ok(picker) = Reflect::get(&window, &JsValue::from_str("showDirectoryPicker")) else {
        return PickDir::Unsupported;
    };
    let Ok(picker_fn) = picker.dyn_into::<Function>() else {
        return PickDir::Unsupported;
    };
    let Ok(promise) = picker_fn.call0(&window) else {
        return PickDir::Cancelled;
    };
    match JsFuture::from(js_sys::Promise::from(promise)).await {
        Ok(handle) => PickDir::Handle(handle),
        Err(_) => PickDir::Cancelled,
    }
}

async fn collect_tree(
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
    fn allows_only_listed_extensions() {
        assert!(extension_allowed("help.md"));
        assert!(extension_allowed("data.json5"));
        assert!(extension_allowed("views/simple-front.html"));
        assert!(!extension_allowed("print.pdf"));
        assert!(!extension_allowed("notes.txt"));
        assert!(!extension_allowed("LICENSE"));
    }

    #[test]
    fn unique_folder_gets_suffix() {
        let taken = |name: &str| name == "demo" || name == "demo-1";
        assert_eq!(unique_folder_name("demo", taken), "demo-2");
        assert_eq!(unique_folder_name("fresh", taken), "fresh");
    }
}
