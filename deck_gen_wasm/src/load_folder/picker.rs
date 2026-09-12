//! Browser directory picker: File System Access API, else `<input webkitdirectory>`.
//!
//! Feature-detects `showDirectoryPicker`. A missing API or a failed call falls
//! through to a hidden file input. The chosen handle/list is then walked by
//! [`super::read`].

use js_sys::{Function, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileList;

use super::input::pick_with_hidden_input;
use super::read::{collect_from_file_list, collect_tree, CollectedEntries, CollectedFolder};
use super::PickResult;
use crate::conf;

/// Open a directory picker and read allowed files (or a reject/cancel).
pub async fn pick_and_read_folder() -> PickResult {
    match pick_directory().await {
        PickDir::Cancelled => PickResult::Cancelled,
        PickDir::Handle(handle) => {
            let name = Reflect::get(&handle, &JsValue::from_str("name"))
                .ok()
                .and_then(|value| value.as_string())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| conf::import::DEFAULT_FOLDER_NAME.to_string());
            finish_collect(name, collect_tree(&handle, "").await)
        }
        PickDir::FileList(list) => finish_collect_from_list(collect_from_file_list(list).await),
    }
}

fn finish_collect(name: String, result: Result<CollectedEntries, String>) -> PickResult {
    match result {
        Ok(entries) => finish_entries(name, entries),
        Err(err) => PickResult::Rejected(err),
    }
}

fn finish_collect_from_list(result: Result<CollectedFolder, String>) -> PickResult {
    match result {
        Ok(folder) => finish_entries(folder.name, folder.entries),
        Err(err) => PickResult::Rejected(err),
    }
}

fn finish_entries(name: String, entries: CollectedEntries) -> PickResult {
    if let Some(reason) = entries.reject_reason() {
        PickResult::Rejected(reason)
    } else {
        PickResult::Ready {
            name,
            files: entries.files,
            dirs: entries.dirs,
        }
    }
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
    match pick_with_hidden_input(|input| {
        input.set_multiple(true);
        let _ = input.set_attribute("webkitdirectory", "");
        let _ = input.set_attribute("directory", "");
    })
    .await
    {
        Some(list) => PickDir::FileList(list),
        None => PickDir::Cancelled,
    }
}
