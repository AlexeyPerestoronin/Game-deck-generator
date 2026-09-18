//! Activity-bar control that opens the default mail client via mailto: for feedback.

use leptos::prelude::*;

use crate::icons::FeedbackIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_feedback::send_feedback_via_email;

/// Open default email client with prefilled feedback (from conf).
#[component]
pub fn FeedbackButton() -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_FEEDBACK).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                aria-label=move || locale::localize(keys::ARIA_FEEDBACK)
                on:click=move |_| { let _ = send_feedback_via_email(); }
            >
                <FeedbackIcon />
            </button>
        </DelayedTooltip>
    }
}
