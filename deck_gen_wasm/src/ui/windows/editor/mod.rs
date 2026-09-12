//! Open-file pane: tab strip plus source editor or HTML/Markdown/PDF/image preview.
//!
//! Tabs come from [`Workspace::tabs`](crate::workspace::Workspace::tabs), or
//! from [`Workspace::preview_tabs`](crate::workspace::Workspace::preview_tabs)
//! when split-for-preview is on. Markdown / JSON / HTML / SCSS use syntect
//! HTML behind a transparent textarea; everything else is a plain `<textarea>`.
//! Preview tabs render HTML in an iframe, Markdown as HTML, PDF via a blob-URL
//! iframe, and images via a blob-URL `<img>`.

use leptos::prelude::*;

use crate::workspace::Workspace;

mod highlight;
mod highlighted;
mod iframe;
mod pane;
mod plain;
mod preview;
mod tab;

use pane::EditorPane;

/// Center pane: one editor, or two halves when split-for-preview is on.
#[component]
pub fn Editor(workspace: Workspace) -> impl IntoView {
    view! {
        {move || {
            if workspace.split_preview.get() {
                view! {
                    <div class="editor-split">
                        <EditorPane workspace=workspace preview_pane=false />
                        <EditorPane workspace=workspace preview_pane=true />
                    </div>
                }
                .into_any()
            } else {
                view! { <EditorPane workspace=workspace preview_pane=false /> }.into_any()
            }
        }}
    }
}
