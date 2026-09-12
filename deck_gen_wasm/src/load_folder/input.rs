//! Hidden `<input type="file">` used when the File System Access API is absent
//! or when picking individual files.
//!
//! The input is appended to `document.body`, clicked, then removed. A `change`
//! event resolves to the chosen [`FileList`]; a `cancel` event (or an empty
//! list) is treated as the user dismissing the dialog.

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FileList, HtmlInputElement};

/// Click a hidden file input after `configure` (accept, webkitdirectory, …).
pub(super) async fn pick_with_hidden_input(
    configure: impl FnOnce(&HtmlInputElement),
) -> Option<FileList> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let element = document.create_element("input").ok()?;
    let input = element.dyn_into::<HtmlInputElement>().ok()?;
    input.set_type("file");
    configure(&input);
    let _ = input.style().set_property("display", "none");
    if let Some(body) = document.body() {
        let _ = body.append_child(&input);
    }

    let input_for_change = input.clone();
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
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
        Ok(value) if value.is_null() || value.is_undefined() => None,
        Ok(value) => match value.dyn_into::<FileList>() {
            Ok(list) if list.length() == 0 => None,
            Ok(list) => Some(list),
            Err(_) => None,
        },
        Err(_) => None,
    }
}
