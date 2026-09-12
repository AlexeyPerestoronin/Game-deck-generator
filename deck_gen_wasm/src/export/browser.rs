//! Offer ZIP bytes to the user: File System Access picker, else `<a download>`.
//!
//! `showSaveFilePicker` is feature-detected. If it is missing or fails, the
//! same bytes are turned into a Blob URL and a hidden anchor is clicked.
//! Errors from the picker path are swallowed in favor of the fallback so a
//! permission deny still downloads.

use js_sys::{Array, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

/// Save `bytes` as `filename`, preferring the save picker over an anchor click.
pub async fn save_zip_bytes(bytes: Vec<u8>, filename: &str) -> Result<(), String> {
    if has_save_file_picker() && save_with_picker(&bytes, filename).await.is_ok() {
        return Ok(());
    }
    download_via_anchor(&bytes, filename)
}

fn has_save_file_picker() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    Reflect::has(&window, &JsValue::from_str("showSaveFilePicker")).unwrap_or(false)
}

async fn save_with_picker(bytes: &[u8], filename: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let picker = Reflect::get(&window, &JsValue::from_str("showSaveFilePicker"))?;
    let picker_fn = picker.dyn_into::<js_sys::Function>()?;
    let options = picker_options(filename)?;
    let handle = JsFuture::from(js_sys::Promise::from(picker_fn.call1(&window, &options)?)).await?;
    let writable = call_async(&handle, "createWritable", None).await?;
    let data = Uint8Array::from(bytes);
    call_async(&writable, "write", Some(data.into())).await?;
    call_async(&writable, "close", None).await?;
    Ok(())
}

fn picker_options(filename: &str) -> Result<js_sys::Object, JsValue> {
    let zip_ext = Array::new();
    zip_ext.push(&JsValue::from_str(".zip"));
    let accept = js_sys::Object::new();
    Reflect::set(&accept, &JsValue::from_str("application/zip"), &zip_ext)?;

    let type_entry = js_sys::Object::new();
    Reflect::set(
        &type_entry,
        &JsValue::from_str("description"),
        &JsValue::from_str("ZIP archive"),
    )?;
    Reflect::set(&type_entry, &JsValue::from_str("accept"), &accept)?;
    let types = Array::new();
    types.push(&type_entry);

    let options = js_sys::Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("suggestedName"),
        &JsValue::from_str(filename),
    )?;
    Reflect::set(&options, &JsValue::from_str("types"), &types)?;
    Ok(options)
}

async fn call_async(
    receiver: &JsValue,
    method: &str,
    arg: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let func =
        Reflect::get(receiver, &JsValue::from_str(method))?.dyn_into::<js_sys::Function>()?;
    let promise = match arg {
        Some(value) => func.call1(receiver, &value)?,
        None => func.call0(receiver)?,
    };
    JsFuture::from(js_sys::Promise::from(promise)).await
}

fn download_via_anchor(bytes: &[u8], filename: &str) -> Result<(), String> {
    let array = Uint8Array::from(bytes);
    let parts = Array::new();
    parts.push(&array);
    let blob_opts = BlobPropertyBag::new();
    blob_opts.set_type("application/zip");
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &blob_opts)
        .map_err(|_| "Could not build ZIP blob".to_string())?;
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Could not create download URL".to_string())?;
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let document = window.document().ok_or_else(|| "no document".to_string())?;
    let anchor = document
        .create_element("a")
        .map_err(|_| "Could not create download link".to_string())?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| "download link is not an anchor".to_string())?;
    anchor.set_href(&url);
    anchor.set_download(filename);
    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    } else {
        anchor.click();
    }
    let _ = Url::revoke_object_url(&url);
    Ok(())
}
