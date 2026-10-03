//! Browser JS FFI, HTTP GET/POST, and bounded async join.

mod http;
mod js;
mod task;

pub use crate::http::{fetch_bytes, fetch_text, post_json};
pub use crate::js::{
    blob_url, call0, call_async, call_async_js, has_window_fn, is_firefox, revoke_object_url,
};
pub use crate::task::map_join;
