//! Window/JS helpers, text fetch, and bounded `join_all`.

pub use crate::http::fetch_text;
pub use crate::js::{
    blob_url, call0, call_async, call_async_js, has_window_fn, is_firefox, revoke_object_url,
};
pub use crate::task::map_join;
