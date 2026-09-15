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
use super::{PickOutcome, PickResult, PickedFolder};
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_browser as js;

/// Open a directory picker and read allowed files (or a reject/cancel).
pub async fn pick_and_read_folder() -> PickResult {
    match pick_directory().await {
        PickDir::Cancelled => PickOutcome::Cancelled,
        PickDir::Handle(handle) => {
            let name = Reflect::get(&handle, &JsValue::from_str("name"))
                .ok()
                .and_then(|value| value.as_string())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| conf::import::DEFAULT_FOLDER_NAME.to_string());
            finish_entries(name, collect_tree(&handle, "").await)
        }
        PickDir::FileList(list) => match collect_from_file_list(list).await {
            Ok(CollectedFolder { name, entries }) => finish_entries(name, Ok(entries)),
            Err(err) => PickOutcome::Rejected(err),
        },
    }
}

fn finish_entries(name: String, result: Result<CollectedEntries, String>) -> PickResult {
    match result {
        Ok(entries) => {
            if let Some(reason) = entries.reject_reason() {
                PickOutcome::Rejected(reason)
            } else {
                PickOutcome::Ready(PickedFolder {
                    name,
                    files: entries.files,
                    dirs: entries.dirs,
                })
            }
        }
        Err(err) => PickOutcome::Rejected(err),
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
    if !js::has_window_fn("showDirectoryPicker") {
        return None;
    }
    let window = web_sys::window()?;
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
