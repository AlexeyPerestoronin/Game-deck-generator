//! Activity-bar toggle that splits the editor: files left, previews right.

use leptos::prelude::*;

use crate::icons::SplitPreviewIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Split the working area so preview tabs occupy the right half.
#[component]
pub fn SplitPreviewButton(workspace: Workspace) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_SPLIT_PREVIEW).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                class:active=move || workspace.split_preview.get()
                aria-label=move || locale::localize(keys::ARIA_SPLIT_PREVIEW)
                on:click=move |_| workspace.toggle_split_preview()
            >
                <SplitPreviewIcon />
            </button>
        </DelayedTooltip>
    }
}
