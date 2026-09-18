//! Shared JS interop: window function detect, method calls, and Blob URLs.
//!
//! File System Access (`showDirectoryPicker` / `showSaveFilePicker`), ZIP
//! download, HTML/PDF/image preview, and the directory-handle walk all need
//! the same `Reflect` + `Promise` and `Uint8Array` → Blob patterns. Callers
//! still own picker options and error text.

use js_sys::{Array, Function, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, Url};

/// Whether `window[name]` exists (File System Access feature detect).
pub fn has_window_fn(name: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    Reflect::has(&window, &JsValue::from_str(name)).unwrap_or(false)
}

/// `receiver[method]()` and return the value (sync).
pub fn call0(receiver: &JsValue, method: &str) -> Result<JsValue, String> {
    let func = Reflect::get(receiver, &JsValue::from_str(method))
        .map_err(|_| format!("missing {method}"))?
        .dyn_into::<Function>()
        .map_err(|_| format!("{method} is not a function"))?;
    func.call0(receiver).map_err(|_| format!("{method} failed"))
}

/// `receiver[method]()` when the return value is a Promise.
pub async fn call_async(receiver: &JsValue, method: &str) -> Result<JsValue, String> {
    let promise = call0(receiver, method)?;
    JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|_| format!("{method} failed"))
}

/// `receiver[method]()` or `receiver[method](arg)` as a Promise; JS errors pass through.
pub async fn call_async_js(
    receiver: &JsValue,
    method: &str,
    arg: Option<&JsValue>,
) -> Result<JsValue, JsValue> {
    let func =
        Reflect::get(receiver, &JsValue::from_str(method))?.dyn_into::<js_sys::Function>()?;
    let promise = match arg {
        Some(value) => func.call1(receiver, value)?,
        None => func.call0(receiver)?,
    };
    JsFuture::from(js_sys::Promise::from(promise)).await
}

/// Object URL for `bytes` with the given MIME type.
pub fn blob_url(bytes: &[u8], mime: &str) -> Option<String> {
    let array = Uint8Array::from(bytes);
    let parts = Array::new();
    parts.push(&array);
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts).ok()?;
    Url::create_object_url_with_blob(&blob).ok()
}

/// Revoke a previous [`blob_url`]; no-op on empty string.
pub fn revoke_object_url(url: &str) {
    if !url.is_empty() {
        let _ = Url::revoke_object_url(url);
    }
}

/// Detect Firefox (pdf.js vs Chromium PDF plugin differ in how they apply color-scheme
/// for forcing light page background in PDF preview).
pub fn is_firefox() -> bool {
    let call = Function::new_with_args(
        "",
        "try { return /firefox/i.test((navigator && navigator.userAgent) || ''); } catch(e){ return false; }",
    );
    call.call0(&JsValue::UNDEFINED)
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
