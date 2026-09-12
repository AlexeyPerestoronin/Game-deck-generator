//! Browser directory picker: File System Access API, else `<input webkitdirectory>`.
//!
//! Feature-detects `showDirectoryPicker`. A missing API or a failed call falls
//! through to a hidden file input. The chosen handle/list is then walked by
//! [`super::read`].

use js_sys::{Function, Promise, Reflect};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileList, HtmlInputElement};

use super::policy::reject_message;
use super::read::{collect_from_file_list, collect_tree};
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
        let _ =
            input.add_event_listener_with_callback("cancel", on_cancel.as_ref().unchecked_ref());
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
