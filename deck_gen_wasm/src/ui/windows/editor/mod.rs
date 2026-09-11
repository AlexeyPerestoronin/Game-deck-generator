//! Open-file pane: tab strip plus source editor or HTML/Markdown preview.
//!
//! Tabs come from [`Workspace::tabs`](crate::workspace::Workspace::tabs).
//! Markdown / JSON / HTML / SCSS use syntect HTML behind a transparent
//! textarea; everything else is a plain `<textarea>`. Preview tabs render
//! HTML in an iframe and Markdown as HTML.

use leptos::prelude::*;

use crate::workspace::{TabKind, Workspace};

mod highlight;
mod highlighted;
mod iframe;
mod plain;
mod preview;
mod tab;

use highlight::can_highlight;
use highlighted::HighlightedEditor;
use plain::PlainEditor;
use preview::PreviewPane;
use tab::EditorTab;

/// Center pane: tab strip plus the active editor or preview.
#[component]
pub fn Editor(workspace: Workspace) -> impl IntoView {
    view! {
        <main class="editor">
            <header class="editor-tabs" role="tablist">
                <For
                    each=move || workspace.tabs.get()
                    key=|tab| {
                        let kind = match tab.kind {
                            TabKind::Edit => "edit",
                            TabKind::Preview => "preview",
                        };
                        format!("{kind}:{}", tab.path)
                    }
                    children=move |tab| {
                        view! { <EditorTab workspace=workspace tab=tab /> }
                    }
                />
            </header>
            {move || match workspace.active_tab.get() {
                None => view! {
                    <div class="editor-empty">"Select a file to edit, or create one in the explorer."</div>
                }
                .into_any(),
                Some(tab) if workspace.vfs.get().is_binary(&tab.path) => view! {
                    <div class="editor-empty">
                        {format!(
                            "Binary file ({} bytes).",
                            workspace.vfs.get().read_bytes(&tab.path).map(|b| b.len()).unwrap_or(0)
                        )}
                    </div>
                }
                .into_any(),
                Some(tab) if tab.kind == TabKind::Preview => view! {
                    <PreviewPane workspace=workspace path=tab.path />
                }
                .into_any(),
                Some(tab) if can_highlight(&tab.path) => view! {
                    <HighlightedEditor workspace=workspace path=tab.path />
                }
                .into_any(),
                Some(tab) => view! {
                    <PlainEditor workspace=workspace path=tab.path />
                }
                .into_any(),
            }}
        </main>
    }
}
