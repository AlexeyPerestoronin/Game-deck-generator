//! Activity-bar control that opens the clear-workspace confirm dialog.

use leptos::prelude::*;

use crate::ui::icons::ClearIcon;
use crate::ui::tooltips::DelayedTooltip;

/// Destructive clear; the bar owns the confirm modal.
#[component]
pub fn ClearButton(#[prop(into)] on_open: Callback<()>) -> impl IntoView {
    view! {
        <DelayedTooltip text="Clear the workspace in this browser.">
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
