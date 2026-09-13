//! Overlay editor: syntect HTML in a `<pre>` plus a transparent textarea.

use gloo_timers::future::TimeoutFuture;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlTextAreaElement;

use super::highlight::highlight_html;
use crate::conf;
use crate::html_escape::text_to_html;
use crate::workspace::Workspace;

#[component]
pub(super) fn HighlightedEditor(workspace: Workspace, path: String) -> impl IntoView {
    let pre_ref = NodeRef::<html::Pre>::new();
    let path_for_value = path.clone();
    let path_for_html = path.clone();
    let path_for_input = path;

    let highlight = RwSignal::new(workspace.vfs.with_untracked(|vfs| {
        let text = vfs.read_file(&path_for_html).unwrap_or("");
        highlight_html(&path_for_html, text).unwrap_or_else(|| text_to_html(text))
    }));
    let generation = RwSignal::new(0u32);
    Effect::new(move |_| {
        workspace.vfs.with(|_| {});
        let path = path_for_html.clone();
        let token = generation.get_untracked().wrapping_add(1);
        generation.set(token);
        spawn_local(async move {
            TimeoutFuture::new(conf::ui::HIGHLIGHT_DEBOUNCE_MS).await;
            if generation.get_untracked() != token {
                return;
            }
            let html = workspace.vfs.with_untracked(|vfs| {
                let text = vfs.read_file(&path).unwrap_or("");
                highlight_html(&path, text).unwrap_or_else(|| text_to_html(text))
            });
            highlight.set(html);
        });
    });

    view! {
        <div class="code-editor">
            <pre
                node_ref=pre_ref
                class="code-highlight"
                inner_html=move || highlight.get()
            ></pre>
            <textarea
                class="editor-area code-input"
                spellcheck="false"
                prop:value=move || {
                    workspace
                        .vfs
                        .with(|vfs| vfs.read_file(&path_for_value).map(str::to_string))
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
