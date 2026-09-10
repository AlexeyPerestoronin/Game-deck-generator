use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

use super::highlight::{can_highlight, highlight_html};
use crate::fs::file_name;
use crate::workspace::Workspace;

#[component]
pub fn Editor(workspace: Workspace) -> impl IntoView {
    let open_file = move || workspace.open_file_path();

    view! {
        <main class="editor">
            <header class="editor-tab">
                {move || match open_file() {
                    Some(path) => file_name(&path).to_string(),
                    None => "No file open".into(),
                }}
            </header>
            {move || match open_file() {
                None => view! {
                    <div class="editor-empty">"Select a file to edit, or create one in the explorer."</div>
                }
                .into_any(),
                Some(path) if can_highlight(&path) => view! {
                    <HighlightedEditor workspace=workspace path=path />
                }
                .into_any(),
                Some(path) => view! {
                    <PlainEditor workspace=workspace path=path />
                }
                .into_any(),
            }}
        </main>
    }
}

#[component]
fn PlainEditor(workspace: Workspace, path: String) -> impl IntoView {
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

#[component]
fn HighlightedEditor(workspace: Workspace, path: String) -> impl IntoView {
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
