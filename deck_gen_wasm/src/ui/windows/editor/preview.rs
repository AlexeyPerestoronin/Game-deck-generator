//! Rendered preview of HTML (`srcdoc` iframe) or Markdown (inner HTML).

use leptos::prelude::*;

use super::iframe::inline_relative_iframes;
use crate::fs::file_ext;
use crate::workspace::Workspace;

#[component]
pub(super) fn PreviewPane(workspace: Workspace, path: String) -> impl IntoView {
    let ext = file_ext(&path).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => view! { <HtmlPreview workspace=workspace path=path /> }.into_any(),
        _ => view! { <MarkdownPreview workspace=workspace path=path /> }.into_any(),
    }
}

#[component]
fn HtmlPreview(workspace: Workspace, path: String) -> impl IntoView {
    view! {
        <iframe
            class="preview-frame"
            prop:srcdoc=move || {
                let vfs = workspace.vfs.get();
                let Some(html) = vfs.read_file(&path) else {
                    return String::new();
                };
                inline_relative_iframes(&vfs, &path, html)
            }
        />
    }
}

#[component]
fn MarkdownPreview(workspace: Workspace, path: String) -> impl IntoView {
    view! {
        <div
            class="preview-md"
            inner_html=move || {
                let vfs = workspace.vfs.get();
                markdown_to_html(vfs.read_file(&path).unwrap_or(""))
            }
        ></div>
    }
}

fn markdown_to_html(src: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    let parser = pulldown_cmark::Parser::new_ext(src, options);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}
