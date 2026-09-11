//! One editor tab: label, activate on click, close without activating neighbors.

use leptos::prelude::*;

use crate::fs::file_name;
use crate::workspace::{OpenTab, TabKind, Workspace};

#[component]
pub(super) fn EditorTab(workspace: Workspace, tab: OpenTab) -> impl IntoView {
    let tab_for_active = tab.clone();
    let tab_for_click = tab.clone();
    let tab_for_close = tab.clone();
    let label = match tab.kind {
        TabKind::Edit => file_name(&tab.path).to_string(),
        TabKind::Preview => format!("Preview {}", file_name(&tab.path)),
    };
    view! {
        <div
            class="editor-tab"
            class:active=move || workspace.active_tab.get().as_ref() == Some(&tab_for_active)
            role="tab"
            title=tab.path.clone()
            on:click=move |_| workspace.activate_tab(tab_for_click.clone())
        >
            <span class="editor-tab-label">{label}</span>
            <button
                class="editor-tab-close"
                type="button"
                title="Close"
                on:click=move |ev| {
                    ev.stop_propagation();
                    workspace.close_tab(tab_for_close.clone());
                }
            >
                "×"
            </button>
        </div>
    }
}
