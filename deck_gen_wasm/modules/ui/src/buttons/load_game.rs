//! Activity-bar control that opens the load-folder confirm dialog.

use leptos::prelude::*;

use crate::icons::LoadGameIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Ask the user to confirm, then pick a disk folder into `games/`.
#[component]
pub fn LoadGameButton(workspace: Workspace, #[prop(into)] on_open: Callback<()>) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_LOAD_GAME).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                aria-label=move || locale::localize(keys::ARIA_LOAD_GAME)
                disabled=move || workspace.loading.get()
                on:click=move |_| on_open.run(())
            >
                <LoadGameIcon />
            </button>
        </DelayedTooltip>
    }
}
