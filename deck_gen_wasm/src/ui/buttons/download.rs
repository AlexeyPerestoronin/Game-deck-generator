//! Activity-bar control that encodes the VFS as a ZIP download.

use leptos::prelude::*;

use crate::ui::icons::DownloadIcon;
use crate::ui::tooltips::DelayedTooltip;
use crate::workspace::Workspace;

/// Offer the workspace as `workspace.zip`.
#[component]
pub fn DownloadButton(workspace: Workspace) -> impl IntoView {
    view! {
        <DelayedTooltip text="Download the workspace as a ZIP archive.">
            <button
                class="activity-btn"
                aria-label="Download"
                on:click=move |_| workspace.download()
            >
                <DownloadIcon />
            </button>
        </DelayedTooltip>
    }
}
