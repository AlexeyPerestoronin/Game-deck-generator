//! Activity-bar control that opens the clear-workspace confirm dialog.

use leptos::prelude::*;

use crate::icons::ClearIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_conf as conf;

/// Destructive clear; the bar owns the confirm modal.
#[component]
pub fn ClearButton(#[prop(into)] on_open: Callback<()>) -> impl IntoView {
    view! {
        <DelayedTooltip text=conf::ui::TOOLTIP_CLEAR>
            <button
                class="activity-btn"
                aria-label="Clear"
                on:click=move |_| on_open.run(())
            >
                <ClearIcon />
            </button>
        </DelayedTooltip>
    }
}
