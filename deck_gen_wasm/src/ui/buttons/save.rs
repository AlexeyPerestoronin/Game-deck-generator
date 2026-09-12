//! Activity-bar control that writes the workspace snapshot to this browser.

use leptos::prelude::*;

use crate::ui::icons::SaveIcon;
use crate::ui::tooltips::DelayedTooltip;
use crate::workspace::Workspace;

/// Persist the current session in this browser.
#[component]
pub fn SaveButton(workspace: Workspace) -> impl IntoView {
    view! {
        <DelayedTooltip text="Save the workspace in this browser. A page reload will restore it.">
            <button
                class="activity-btn"
                aria-label="Save"
                on:click=move |_| workspace.persist()
            >
                <SaveIcon />
            </button>
        </DelayedTooltip>
    }
}
