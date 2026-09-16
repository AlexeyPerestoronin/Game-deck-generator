//! Activity-bar control that opens the clear-workspace confirm dialog.

use leptos::prelude::*;

use crate::icons::ClearIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

/// Destructive clear; the bar owns the confirm modal.
#[component]
pub fn ClearButton(#[prop(into)] on_open: Callback<()>) -> impl IntoView {
    // Snapshot at mount for & 'static expected by DelayedTooltip (no edit of tooltip component).
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_CLEAR).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                aria-label=move || locale::localize(keys::ARIA_CLEAR)
                on:click=move |_| on_open.run(())
            >
                <ClearIcon />
            </button>
        </DelayedTooltip>
    }
}
