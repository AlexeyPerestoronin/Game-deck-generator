//! Activity-bar toggle that splits the editor: files left, previews right.

use leptos::prelude::*;

use crate::icons::SplitPreviewIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_workspace::Workspace;

/// Split the working area so preview tabs occupy the right half.
#[component]
pub fn SplitPreviewButton(workspace: Workspace) -> impl IntoView {
    view! {
        <DelayedTooltip text="Split for preview. Files stay on the left; previews open on the right.">
            <button
                class="activity-btn"
                class:active=move || workspace.split_preview.get()
                aria-label="Split for preview"
                on:click=move |_| workspace.toggle_split_preview()
            >
                <SplitPreviewIcon />
            </button>
        </DelayedTooltip>
    }
}
