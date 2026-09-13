//! Unhighlighted `<textarea>` bound to a UTF-8 VFS file.
//!
//! The DOM holds the text while typing. VFS is updated on idle, blur, or
//! unmount so explorer / preview / autosave are not notified per keystroke.

use leptos::html;
use leptos::prelude::*;

use super::draft;
use crate::workspace::Workspace;

#[component]
pub(super) fn PlainEditor(workspace: Workspace, path: String) -> impl IntoView {
    let initial = workspace
        .vfs
        .with_untracked(|vfs| vfs.read_file(&path).unwrap_or("").to_string());
    let area_ref = NodeRef::<html::Textarea>::new();
    let generation = RwSignal::new(0u32);
    let path_input = path.clone();
    let path_blur = path.clone();
    let path_cleanup = path;
    let primed = RwSignal::new(false);
    Effect::new(move |_| {
        let Some(area) = area_ref.get() else {
            return;
        };
        if primed.get_untracked() {
            return;
        }
        area.set_value(&initial);
        primed.set(true);
    });
    on_cleanup(move || draft::commit(workspace, &path_cleanup, area_ref.get_untracked()));

    view! {
        <textarea
            node_ref=area_ref
            class="editor-area"
            spellcheck="false"
            on:input=move |ev| {
                draft::note(workspace, path_input.clone(), event_target_value(&ev), generation);
            }
            on:blur=move |_| draft::commit(workspace, &path_blur, area_ref.get_untracked())
        />
    }
}
