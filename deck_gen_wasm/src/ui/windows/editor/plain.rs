//! Unhighlighted `<textarea>` bound to a UTF-8 VFS file.

use leptos::prelude::*;

use crate::workspace::Workspace;

#[component]
pub(super) fn PlainEditor(workspace: Workspace, path: String) -> impl IntoView {
    let path_for_value = path.clone();
    let path_for_input = path;
    view! {
        <textarea
            class="editor-area"
            prop:value=move || {
                workspace
                    .vfs
                    .get()
                    .read_file(&path_for_value)
                    .map(str::to_string)
                    .unwrap_or_default()
            }
            on:input=move |ev| {
                let value = event_target_value(&ev);
                workspace.vfs.update(|vfs| {
                    let _ = vfs.write_file(&path_for_input, value);
                });
            }
        />
    }
}
