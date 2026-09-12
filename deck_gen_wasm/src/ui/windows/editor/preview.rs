//! Rendered preview of HTML (`srcdoc` iframe), Markdown (inner HTML),
//! PDF (blob URL iframe), or images (blob URL `<img>`).

use js_sys::{Array, Uint8Array};
use leptos::prelude::*;
use web_sys::{Blob, BlobPropertyBag, Url};

use super::iframe::inline_relative_iframes;
use crate::fs::file_ext;
use crate::workspace::Workspace;

/// Active-tab preview: HTML, Markdown, PDF, or image, chosen by file extension.
#[component]
pub(super) fn PreviewPane(workspace: Workspace, path: String) -> impl IntoView {
    let ext = file_ext(&path).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => view! { <HtmlPreview workspace=workspace path=path /> }.into_any(),
        "pdf" => view! { <PdfPreview workspace=workspace path=path /> }.into_any(),
        "jpg" | "jpeg" | "png" | "ico" | "icon" => {
            view! { <ImagePreview workspace=workspace path=path /> }.into_any()
        }
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

#[component]
fn PdfPreview(workspace: Workspace, path: String) -> impl IntoView {
    let bytes = Memo::new(move |_| workspace.vfs.get().read_bytes(&path).map(Vec::from));
    let src = RwSignal::new(String::new());
    Effect::new(move |_| {
        let next = bytes
            .get()
            .as_deref()
            .and_then(pdf_blob_url)
            .unwrap_or_default();
        replace_object_url(src, next);
    });
    on_cleanup(move || revoke_object_url(&src.get_untracked()));
    view! {
        <iframe class="preview-frame" prop:src=move || src.get() />
    }
}

#[component]
fn ImagePreview(workspace: Workspace, path: String) -> impl IntoView {
    let mime = image_mime(&path);
    let alt = path.clone();
    let bytes = Memo::new(move |_| workspace.vfs.get().read_bytes(&path).map(Vec::from));
    let src = RwSignal::new(String::new());
    Effect::new(move |_| {
        let next = bytes
            .get()
            .as_deref()
            .and_then(|data| blob_url(data, mime))
            .unwrap_or_default();
        replace_object_url(src, next);
    });
    on_cleanup(move || revoke_object_url(&src.get_untracked()));
    view! {
        <div class="preview-image-wrap">
            <img class="preview-image" prop:src=move || src.get() alt=alt />
        </div>
    }
}

fn image_mime(path: &str) -> &'static str {
    match file_ext(path).unwrap_or("").to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "ico" | "icon" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn pdf_blob_url(bytes: &[u8]) -> Option<String> {
    blob_url(bytes, "application/pdf")
}

fn blob_url(bytes: &[u8], mime: &str) -> Option<String> {
    let array = Uint8Array::from(bytes);
    let parts = Array::new();
    parts.push(&array);
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts).ok()?;
    Url::create_object_url_with_blob(&blob).ok()
}

fn replace_object_url(slot: RwSignal<String>, next: String) {
    let prev = slot.get_untracked();
    slot.set(next.clone());
    if prev != next {
        revoke_object_url(&prev);
    }
}

fn revoke_object_url(url: &str) {
    if !url.is_empty() {
        let _ = Url::revoke_object_url(url);
    }
}
