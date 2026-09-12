//! One editor half: tab strip plus the active source view or preview.
//!
//! The body is a one-item [`For`] keyed by tab identity so Leptos remounts
//! the editor or preview when the active tab changes. Without that, two
//! `PreviewPane`s of the same type are reconciled in place and keep showing
//! the file that was opened last (iframes and blob URLs do not pick up a
//! new `path` prop).

use leptos::prelude::*;

use super::highlight::can_highlight;
use super::highlighted::HighlightedEditor;
use super::plain::PlainEditor;
use super::preview::PreviewPane;
use super::tab::EditorTab;
use crate::workspace::{OpenTab, TabKind, Workspace};

/// One tab strip plus the active editor or preview for that strip.
#[component]
pub(super) fn EditorPane(workspace: Workspace, preview_pane: bool) -> impl IntoView {
    let empty = if preview_pane {
        "No preview open."
    } else {
        "Select a file to edit, or create one in the explorer."
    };
    view! {
        <main class="editor">
            <header class="editor-tabs" role="tablist">
                <For
                    each=move || {
                        if preview_pane {
                            workspace.preview_tabs.get()
                        } else {
                            workspace.tabs.get()
                        }
                    }
                    key=tab_key
                    children=move |tab| {
                        view! {
                            <EditorTab workspace=workspace tab=tab preview_pane=preview_pane />
                        }
                    }
                />
            </header>
            <div class="editor-body">
                <For
                    each=move || {
                        let active = if preview_pane {
                            workspace.active_preview_tab.get()
                        } else {
                            workspace.active_tab.get()
                        };
                        vec![active]
                    }
                    key=body_key
                    children=move |tab| pane_body(workspace, tab, empty)
                />
            </div>
        </main>
    }
}

fn pane_body(workspace: Workspace, active: Option<OpenTab>, empty: &'static str) -> AnyView {
    match active {
        None => view! { <div class="editor-empty">{empty}</div> }.into_any(),
        Some(tab) if tab.kind == TabKind::Preview => view! {
            <PreviewPane workspace=workspace path=tab.path />
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
        Some(tab) if can_highlight(&tab.path) => view! {
            <HighlightedEditor workspace=workspace path=tab.path />
        }
        .into_any(),
        Some(tab) => view! {
            <PlainEditor workspace=workspace path=tab.path />
        }
        .into_any(),
    }
}

fn tab_key(tab: &OpenTab) -> String {
    let kind = match tab.kind {
        TabKind::Edit => "edit",
        TabKind::Preview => "preview",
    };
    format!("{kind}:{}", tab.path)
}

fn body_key(tab: &Option<OpenTab>) -> String {
    match tab {
        None => "empty".to_string(),
        Some(tab) => tab_key(tab),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_key_changes_with_path_and_kind() {
        let edit = OpenTab {
            path: "a.md".into(),
            kind: TabKind::Edit,
        };
        let preview_a = OpenTab {
            path: "a.md".into(),
            kind: TabKind::Preview,
        };
        let preview_b = OpenTab {
            path: "b.md".into(),
            kind: TabKind::Preview,
        };
        assert_ne!(
            body_key(&Some(preview_a.clone())),
            body_key(&Some(preview_b))
        );
        assert_ne!(body_key(&Some(edit)), body_key(&Some(preview_a)));
        assert_eq!(body_key(&None), "empty");
    }
}
