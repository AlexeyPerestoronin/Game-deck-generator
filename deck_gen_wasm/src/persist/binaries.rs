//! IndexedDB copy of VFS binary files (PDF and images).
//!
//! localStorage holds the text tree only. This module stores one record
//! (`files`) whose value is a JS object of workspace path → `Uint8Array`.
//! Open/put/get are async; callers `spawn_local`. A cheap fingerprint lets
//! the UI skip a rewrite when only text files changed.

use js_sys::{Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};

use crate::conf;

/// Stable hash of binary path/length/edges so autosave can skip unchanged blobs.
pub fn binaries_fingerprint(entries: &[(String, Vec<u8>)]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for (path, data) in entries {
        for byte in path.as_bytes() {
            h ^= u64::from(*byte);
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= data.len() as u64;
        h = h.wrapping_mul(0x100000001b3);
        if let Some(&byte) = data.first() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x100000001b3);
        }
        if let Some(&byte) = data.last() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    h
}

/// Read every stored binary, or an empty list if the DB/record is missing.
pub async fn load_binaries() -> Result<Vec<(String, Vec<u8>)>, String> {
    let db = open_db().await?;
    let tx = db
        .transaction_with_str(conf::session::IDB_STORE)
        .map_err(|_| "indexedDB transaction failed".to_string())?;
    let store = tx
        .object_store(conf::session::IDB_STORE)
        .map_err(|_| "indexedDB store missing".to_string())?;
    let request = store
        .get(&JsValue::from_str(conf::session::IDB_KEY))
        .map_err(|_| "indexedDB get failed".to_string())?;
    let value = await_request(&request).await?;
    if value.is_undefined() || value.is_null() {
        return Ok(Vec::new());
    }
    Ok(decode_binaries(&value))
}

/// Replace the stored binary map with `entries` (empty list clears them).
pub async fn save_binaries(entries: &[(String, Vec<u8>)]) -> Result<(), String> {
    let db = open_db().await?;
    let tx = db
        .transaction_with_str_and_mode(conf::session::IDB_STORE, IdbTransactionMode::Readwrite)
        .map_err(|_| "indexedDB transaction failed".to_string())?;
    let store = tx
        .object_store(conf::session::IDB_STORE)
        .map_err(|_| "indexedDB store missing".to_string())?;
    let value = encode_binaries(entries);
    let request = store
        .put_with_key(&value, &JsValue::from_str(conf::session::IDB_KEY))
        .map_err(|_| "indexedDB put failed".to_string())?;
    await_request(&request).await?;
    Ok(())
}

async fn open_db() -> Result<IdbDatabase, String> {
    let window = web_sys::window().ok_or_else(|| "window is missing".to_string())?;
    let factory = window
        .indexed_db()
        .map_err(|_| "indexedDB is not available".to_string())?
        .ok_or_else(|| "indexedDB is not available".to_string())?;
    let request: IdbOpenDbRequest = factory
        .open_with_u32(conf::session::IDB_NAME, conf::session::IDB_VERSION)
        .map_err(|_| "indexedDB open failed".to_string())?;
    let on_upgrade = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        let Some(target) = event.target() else {
            return;
        };
        let Ok(req) = target.dyn_into::<IdbOpenDbRequest>() else {
            return;
        };
        let Ok(result) = req.result() else {
            return;
        };
        let Ok(db) = result.dyn_into::<IdbDatabase>() else {
            return;
        };
        let names = db.object_store_names();
        if !names.contains(conf::session::IDB_STORE) {
            let _ = db.create_object_store(conf::session::IDB_STORE);
        }
    });
    request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
    on_upgrade.forget();
    let value = await_request(request.unchecked_ref()).await?;
    value
        .dyn_into::<IdbDatabase>()
        .map_err(|_| "indexedDB open did not return a database".to_string())
}

fn encode_binaries(entries: &[(String, Vec<u8>)]) -> Object {
    let obj = Object::new();
    for (path, data) in entries {
        let array = Uint8Array::new_with_length(data.len() as u32);
        array.copy_from(data);
        let _ = Reflect::set(&obj, &JsValue::from_str(path), &array);
    }
    obj
}

fn decode_binaries(value: &JsValue) -> Vec<(String, Vec<u8>)> {
    let Ok(obj) = value.clone().dyn_into::<Object>() else {
        return Vec::new();
    };
    let keys = Object::keys(&obj);
    let mut out = Vec::new();
    for index in 0..keys.length() {
        let key = keys.get(index);
        let Some(path) = key.as_string() else {
            continue;
        };
        let Ok(raw) = Reflect::get(&obj, &key) else {
            continue;
        };
        let array = Uint8Array::new(&raw);
        let mut data = vec![0u8; array.length() as usize];
        array.copy_to(&mut data);
        out.push((path, data));
    }
    out
}

async fn await_request(request: &IdbRequest) -> Result<JsValue, String> {
    let request = request.clone();
    let promise = Promise::new(&mut |resolve, reject| {
        let req_ok = request.clone();
        let resolve_ok = resolve.clone();
        let reject_ok = reject.clone();
        let on_success = Closure::<dyn FnMut()>::once(move || match req_ok.result() {
            Ok(value) => {
                let _ = resolve_ok.call1(&JsValue::NULL, &value);
            }
            Err(err) => {
                let _ = reject_ok.call1(&JsValue::NULL, &err);
            }
        });
        let on_error = Closure::<dyn FnMut()>::once(move || {
            let _ = reject.call1(
                &JsValue::NULL,
                &JsValue::from_str("indexedDB request failed"),
            );
        });
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_success.forget();
        on_error.forget();
    });
    JsFuture::from(promise)
        .await
        .map_err(|_| "indexedDB request failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_changes_when_bytes_or_path_change() {
        let a = vec![("a.pdf".into(), vec![1, 2, 3])];
        let b = vec![("a.pdf".into(), vec![1, 2, 3])];
        let c = vec![("a.pdf".into(), vec![1, 2, 4])];
        let d = vec![("b.pdf".into(), vec![1, 2, 3])];
        assert_eq!(binaries_fingerprint(&a), binaries_fingerprint(&b));
        assert_ne!(binaries_fingerprint(&a), binaries_fingerprint(&c));
        assert_ne!(binaries_fingerprint(&a), binaries_fingerprint(&d));
        assert_eq!(binaries_fingerprint(&[]), binaries_fingerprint(&[]));
    }
}
