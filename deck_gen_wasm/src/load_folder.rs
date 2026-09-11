//! Pick a local folder and copy it into the workspace.
//!
//! Only `md` / `json` / `json5` / `html` / `scss` files are accepted; any other
//! extension rejects the whole import. The picker uses `showDirectoryPicker`
//! when present, otherwise a hidden `<input webkitdirectory>`. The folder is
//! copied under `games/` with a unique name.

use std::collections::BTreeSet;

use js_sys::{Array, Function, Promise, Reflect};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileList, HtmlInputElement};

use crate::fs::{file_ext, join_path, parent_path, unique_name, Vfs};

const ALLOWED: &[&str] = &["md", "json", "json5", "html", "scss"];

/// Outcome of the directory picker, including extension-policy rejects.
pub enum PickResult {
    /// User dismissed the picker.
    Cancelled,
    /// I/O failure or disallowed extensions.
    Rejected(String),
    /// Folder name plus relative files and directories.
    Ready { name: String, files: Vec<(String, String)>, dirs: Vec<String> },
}

/// Whether `path` has an allowed source-file extension.
pub fn extension_allowed(path: &str) -> bool {
    match file_ext(path).map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) => ALLOWED.iter().any(|ok| ext == *ok),
        None => false,
    }
}

/// Unique sibling under `games/`; empty `base` becomes `"game"`.
pub fn unique_folder_name(base: &str, taken: impl Fn(&str) -> bool) -> String {
    let base = if base.is_empty() { "game" } else { base };
    unique_name(base, taken)
}

/// Copy `dirs` / `files` into `games/{unique}` and return that folder name.
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

/// Open a directory picker and read allowed files (or a reject/cancel).
pub async fn pick_and_read_folder() -> PickResult {
    match pick_directory().await {
        PickDir::Cancelled => PickResult::Cancelled,
        PickDir::Handle(handle) => {
            let name = Reflect::get(&handle, &JsValue::from_str("name"))
                .ok()
                .and_then(|value| value.as_string())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "game".to_string());
            finish_collect(name, collect_tree(&handle, "").await)
        }
        PickDir::FileList(list) => finish_collect_from_list(collect_from_file_list(list).await),
    }
}

fn finish_collect(
    name: String,
    result: Result<(Vec<(String, String)>, Vec<String>, Vec<String>), String>,
) -> PickResult {
    match result {
        Ok((files, dirs, rejected)) if rejected.is_empty() => {
            PickResult::Ready { name, files, dirs }
        }
        Ok((_, _, rejected)) => PickResult::Rejected(reject_message(&rejected)),
        Err(err) => PickResult::Rejected(err),
    }
}

fn finish_collect_from_list(
    result: Result<(String, Vec<(String, String)>, Vec<String>, Vec<String>), String>,
) -> PickResult {
    match result {
        Ok((name, files, dirs, rejected)) if rejected.is_empty() => {
            PickResult::Ready { name, files, dirs }
        }
        Ok((_, _, _, rejected)) => PickResult::Rejected(reject_message(&rejected)),
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
    Cancelled,
    Handle(JsValue),
    FileList(FileList),
}

async fn pick_directory() -> PickDir {
    if let Some(handle) = pick_with_directory_picker().await {
        return handle;
    }
    pick_with_input().await
}

async fn pick_with_directory_picker() -> Option<PickDir> {
    let window = web_sys::window()?;
    let has_picker =
        Reflect::has(&window, &JsValue::from_str("showDirectoryPicker")).unwrap_or(false);
    if !has_picker {
        return None;
    }
    let picker = Reflect::get(&window, &JsValue::from_str("showDirectoryPicker")).ok()?;
    let picker_fn = picker.dyn_into::<Function>().ok()?;
    let promise = picker_fn.call0(&window).ok()?;
    match JsFuture::from(js_sys::Promise::from(promise)).await {
        Ok(handle) => Some(PickDir::Handle(handle)),
        Err(_) => Some(PickDir::Cancelled),
    }
}

async fn pick_with_input() -> PickDir {
    let Some(window) = web_sys::window() else {
        return PickDir::Cancelled;
    };
    let Some(document) = window.document() else {
        return PickDir::Cancelled;
    };
    let Ok(element) = document.create_element("input") else {
        return PickDir::Cancelled;
    };
    let Ok(input) = element.dyn_into::<HtmlInputElement>() else {
        return PickDir::Cancelled;
    };
    input.set_type("file");
    input.set_multiple(true);
    let _ = input.set_attribute("webkitdirectory", "");
    let _ = input.set_attribute("directory", "");
    let _ = input.style().set_property("display", "none");
    if let Some(body) = document.body() {
        let _ = body.append_child(&input);
    }

    let input_for_change = input.clone();
    let promise = Promise::new(&mut |resolve, _reject| {
        let input_for_change = input_for_change.clone();
        let resolve_change = resolve.clone();
        let on_change = Closure::<dyn FnMut()>::once(move || {
            let files = input_for_change.files();
            let payload = files.map_or(JsValue::NULL, JsValue::from);
            let _ = resolve_change.call1(&JsValue::NULL, &payload);
        });
        let resolve_cancel = resolve.clone();
        let on_cancel = Closure::<dyn FnMut()>::once(move || {
            let _ = resolve_cancel.call1(&JsValue::NULL, &JsValue::NULL);
        });
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        let _ = input
            .add_event_listener_with_callback("cancel", on_cancel.as_ref().unchecked_ref());
        on_change.forget();
        on_cancel.forget();
    });
    input.click();
    let result = JsFuture::from(promise).await;
    if let Some(parent) = input.parent_node() {
        let _ = parent.remove_child(&input);
    }
    match result {
        Ok(value) if value.is_null() || value.is_undefined() => PickDir::Cancelled,
        Ok(value) => match value.dyn_into::<FileList>() {
            Ok(list) if list.length() == 0 => PickDir::Cancelled,
            Ok(list) => PickDir::FileList(list),
            Err(_) => PickDir::Cancelled,
        },
        Err(_) => PickDir::Cancelled,
    }
}

fn split_webkit_path(path: &str) -> (Option<String>, String) {
    let path = path.replace('\\', "/");
    match path.split_once('/') {
        Some((root, rest)) if !root.is_empty() && !rest.is_empty() => {
            (Some(root.to_string()), rest.to_string())
        }
        _ => (None, path),
    }
}

async fn collect_from_file_list(
    list: FileList,
) -> Result<(String, Vec<(String, String)>, Vec<String>, Vec<String>), String> {
    let mut folder_name = "game".to_string();
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
    Ok((
        folder_name,
        files,
        dirs.into_iter().collect(),
        rejected,
    ))
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

    #[test]
    fn splits_webkit_relative_path() {
        assert_eq!(
            split_webkit_path("demo/decks/data.json5"),
            (Some("demo".into()), "decks/data.json5".into())
        );
        assert_eq!(split_webkit_path("help.md"), (None, "help.md".into()));
    }
}
