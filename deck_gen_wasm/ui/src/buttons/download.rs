//! Activity-bar control that encodes the VFS as a ZIP download.

use leptos::prelude::*;

use crate::icons::DownloadIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_workspace::Workspace;

/// Offer the workspace as `workspace.zip`.
#[component]
pub fn DownloadButton(workspace: Workspace) -> impl IntoView {
    view! {
        <DelayedTooltip text=conf::ui::TOOLTIP_DOWNLOAD>
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
