//! Activity-bar control that encodes the VFS as a ZIP download.

use leptos::prelude::*;

use crate::icons::DownloadIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Offer the workspace as `workspace.zip`.
#[component]
pub fn DownloadButton(workspace: Workspace) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_DOWNLOAD).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                aria-label=move || locale::localize(keys::ARIA_DOWNLOAD)
                on:click=move |_| workspace.download()
            >
                <DownloadIcon />
            </button>
        </DelayedTooltip>
    }
}
