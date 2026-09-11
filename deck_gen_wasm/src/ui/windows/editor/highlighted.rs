//! Overlay editor: syntect HTML in a `<pre>` plus a transparent textarea.

use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

use super::highlight::highlight_html;
use crate::workspace::Workspace;

#[component]
pub(super) fn HighlightedEditor(workspace: Workspace, path: String) -> impl IntoView {
    let pre_ref = NodeRef::<html::Pre>::new();
    let path_for_value = path.clone();
    let path_for_html = path.clone();
    let path_for_input = path;

    view! {
        <div class="code-editor">
            <pre
                node_ref=pre_ref
                class="code-highlight"
                inner_html=move || {
                    let vfs = workspace.vfs.get();
                    let text = vfs.read_file(&path_for_html).unwrap_or("");
                    highlight_html(&path_for_html, text).unwrap_or_else(|| html_escape(text))
                }
            ></pre>
            <textarea
                class="editor-area code-input"
                spellcheck="false"
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

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
