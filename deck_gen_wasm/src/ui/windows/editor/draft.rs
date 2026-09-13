//! Commit the focused textarea into VFS after idle, blur, or unmount.
//!
//! Typing updates only the draft. VFS (and therefore explorer, preview,
//! syntect, autosave) is notified once the pause in [`crate::conf::ui::EDIT_FLUSH_MS`]
//! elapses, or immediately on blur / tab teardown.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;

use crate::conf;
use crate::workspace::Workspace;

/// Store `text` as the draft and rewrite VFS after the idle pause.
pub fn note(workspace: Workspace, path: String, text: String, generation: RwSignal<u32>) {
    workspace.set_draft(path, text);
    let token = generation.get_untracked().wrapping_add(1);
    generation.set(token);
    spawn_local(async move {
        TimeoutFuture::new(conf::ui::EDIT_FLUSH_MS).await;
        if generation.get_untracked() != token {
            return;
        }
        workspace.flush_draft();
    });
}

/// Read the textarea (if mounted) into the draft and flush now.
pub fn commit(workspace: Workspace, path: &str, area: Option<HtmlTextAreaElement>) {
    if let Some(area) = area {
        workspace.set_draft(path.to_string(), area.value());
    }
    workspace.flush_draft();
}
