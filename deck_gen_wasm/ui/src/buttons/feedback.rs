//! Activity-bar control that opens the default mail client via mailto: for feedback.

use leptos::prelude::*;

use crate::icons::FeedbackIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_feedback::send_feedback_via_email;

/// Open default email client with prefilled feedback (from conf).
#[component]
pub fn FeedbackButton() -> impl IntoView {
    view! {
        <DelayedTooltip text=conf::ui::TOOLTIP_FEEDBACK>
            <button
                class="activity-btn"
                aria-label="Feedback"
                on:click=move |_| { let _ = send_feedback_via_email(); }
            >
                <FeedbackIcon />
            </button>
        </DelayedTooltip>
    }
}
