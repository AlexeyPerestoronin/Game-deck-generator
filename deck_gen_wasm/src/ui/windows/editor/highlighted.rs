//! Overlay editor: syntect HTML in a `<pre>` plus a transparent textarea.
//!
//! Keystrokes update the overlay with escaped text only. Syntect runs after
//! the draft is flushed into VFS (idle / blur / unmount), not on each input.

use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

use super::draft;
use super::highlight::highlight_html;
use crate::html_escape::text_to_html;
use crate::workspace::Workspace;

#[component]
pub(super) fn HighlightedEditor(workspace: Workspace, path: String) -> impl IntoView {
    let pre_ref = NodeRef::<html::Pre>::new();
    let area_ref = NodeRef::<html::Textarea>::new();
    let initial = workspace
        .vfs
        .with_untracked(|vfs| vfs.read_file(&path).unwrap_or("").to_string());
    let overlay = RwSignal::new(
        highlight_html(&path, &initial).unwrap_or_else(|| text_to_html(&initial)),
    );
    let generation = RwSignal::new(0u32);
    let path_html = path.clone();
    Effect::new(move |_| {
        let path = path_html.clone();
        let html = workspace.vfs.with(|vfs| {
            let text = vfs.read_file(&path).unwrap_or("");
            highlight_html(&path, text).unwrap_or_else(|| text_to_html(text))
        });
        if !workspace.draft_is_ahead(&path) {
            overlay.set(html);
        }
    });

    let path_input = path.clone();
    let path_blur = path.clone();
    let path_cleanup = path.clone();
    let initial_value = initial.clone();
    let primed = RwSignal::new(false);
    Effect::new(move |_| {
        let Some(area) = area_ref.get() else {
            return;
        };
        if primed.get_untracked() {
            return;
        }
        area.set_value(&initial_value);
        primed.set(true);
    });
    on_cleanup(move || draft::commit(workspace, &path_cleanup, area_ref.get_untracked()));

    view! {
        <div class="code-editor">
            <pre
                node_ref=pre_ref
                class="code-highlight"
                inner_html=move || overlay.get()
            ></pre>
            <textarea
                node_ref=area_ref
                class="editor-area code-input"
                spellcheck="false"
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    overlay.set(text_to_html(&value));
                    draft::note(workspace, path_input.clone(), value, generation);
                }
                on:blur=move |_| draft::commit(workspace, &path_blur, area_ref.get_untracked())
                on:scroll=move |ev| {
                    let Ok(area) = ev.target().unwrap().dyn_into::<HtmlTextAreaElement>() else {
                        return;
                    };
                    if let Some(pre) = pre_ref.get() {
                        pre.set_scroll_top(area.scroll_top());
                        pre.set_scroll_left(area.scroll_left());
                    }
                }
            />
        </div>
    }
}
