//! Rendered preview of HTML (`srcdoc` iframe), Markdown (inner HTML),
//! PDF (blob URL iframe), or images (blob URL `<img>`).
//!
//! PDF must be `iframe.src = blob:` — not an `<embed>` inside a srcdoc shell.
//! Nested embed keeps the viewer chrome and white paper but paints raster
//! card images as black rectangles.

use leptos::prelude::*;

use super::iframe::inline_relative_iframes;
use deck_gen_wasm_fs::kind::{self, FileKind};
use deck_gen_wasm_browser as js;
use deck_gen_wasm_workspace::Workspace;

/// Active-tab preview: HTML, Markdown, PDF, or image, chosen by file extension.
#[component]
pub(super) fn PreviewPane(workspace: Workspace, path: String) -> impl IntoView {
    match kind::kind_of(&path) {
        FileKind::Html => view! { <HtmlPreview workspace=workspace path=path /> }.into_any(),
        FileKind::Pdf => view! { <PdfPreview workspace=workspace path=path /> }.into_any(),
        FileKind::Image => view! { <ImagePreview workspace=workspace path=path /> }.into_any(),
        _ => view! { <MarkdownPreview workspace=workspace path=path /> }.into_any(),
    }
}

#[component]
fn HtmlPreview(workspace: Workspace, path: String) -> impl IntoView {
    view! {
        <div class="preview-host">
            <iframe
                class="preview-frame"
                prop:srcdoc=move || {
                    workspace.vfs.with(|vfs| {
                        let Some(html) = vfs.read_file(&path) else {
                            return String::new();
                        };
                        inline_relative_iframes(vfs, &path, html)
                    })
                }
            />
        </div>
    }
}

#[component]
fn MarkdownPreview(workspace: Workspace, path: String) -> impl IntoView {
    view! {
        <div
            class="preview-md"
            inner_html=move || {
                workspace.vfs.with(|vfs| markdown_to_html(vfs.read_file(&path).unwrap_or("")))
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

#[component]
fn PdfPreview(workspace: Workspace, path: String) -> impl IntoView {
    let bytes = Memo::new(move |_| workspace.vfs.with(|vfs| vfs.read_bytes(&path).map(Vec::from)));
    let src = RwSignal::new(String::new());
    Effect::new(move |_| {
        let next = bytes
            .get()
            .as_deref()
            .and_then(|data| js::blob_url(data, "application/pdf"))
            .unwrap_or_default();
        replace_object_url(src, next);
    });
    on_cleanup(move || js::revoke_object_url(&src.get_untracked()));
    view! {
        <div class="preview-host">
            <Show when=move || !src.get().is_empty()>
                <iframe class="preview-frame" prop:src=move || src.get() />
            </Show>
        </div>
    }
}

#[component]
fn ImagePreview(workspace: Workspace, path: String) -> impl IntoView {
    let mime = kind::image_mime(&path);
    let alt = path.clone();
    let bytes = Memo::new(move |_| workspace.vfs.with(|vfs| vfs.read_bytes(&path).map(Vec::from)));
    let src = RwSignal::new(String::new());
    Effect::new(move |_| {
        let next = bytes
            .get()
            .as_deref()
            .and_then(|data| js::blob_url(data, mime))
            .unwrap_or_default();
        replace_object_url(src, next);
    });
    on_cleanup(move || js::revoke_object_url(&src.get_untracked()));
    view! {
        <div class="preview-image-wrap">
            <img class="preview-image" prop:src=move || src.get() alt=alt />
        </div>
    }
}

fn replace_object_url(slot: RwSignal<String>, next: String) {
    let prev = slot.get_untracked();
    slot.set(next.clone());
    if prev != next {
        js::revoke_object_url(&prev);
    }
}
