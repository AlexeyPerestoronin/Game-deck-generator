//! Activity-bar control that opens the clear-workspace confirm dialog.

use leptos::prelude::*;

use crate::buttons::ActivityButton;
use crate::icons::ClearIcon;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

/// Destructive clear; the bar owns the confirm modal.
#[component]
pub fn ClearButton(#[prop(into)] on_open: Callback<()>) -> impl IntoView {
    // Snapshot at mount for & 'static expected by DelayedTooltip (no edit of tooltip component).
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_CLEAR).into_boxed_str());
    let aria = Signal::derive(move || locale::localize(keys::ARIA_CLEAR));

    view! {
        <ActivityButton
            tip=tip
            aria_label=aria
            on_click=move || on_open.run(())
        >
            <ClearIcon />
        </ActivityButton>
    }
}
