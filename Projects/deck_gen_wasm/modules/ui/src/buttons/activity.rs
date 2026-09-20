//! Reusable activity-bar button with delayed tooltip.
//!
//! Reduces boilerplate across clear/download/prepare_*/etc buttons.
//! Callers still provide the leaked &'static tip (to satisfy DelayedTooltip)
//! and icon via children. Full icon paths are kept (no data-driven).

use leptos::prelude::*;

use crate::tooltips::DelayedTooltip;

/// Common wrapper for activity bar buttons (tooltip + .activity-btn).
#[component]
pub fn ActivityButton(
    /// Already leaked tip text (use Box::leak(localize(key).into_boxed_str()))
    tip: &'static str,
    /// Reactive aria-label
    aria_label: Signal<String>,
    /// Optional disabled signal (button disabled while loading etc.)
    #[prop(optional)]
    disabled: Option<Signal<bool>>,
    /// Optional active signal (e.g. split preview on)
    #[prop(optional)]
    active: Option<Signal<bool>>,
    /// Click handler (Send for leptos view macro compatibility)
    on_click: impl Fn() + Clone + 'static + Send,
    children: Children,
) -> impl IntoView {
    let btn_class = move || {
        let mut c = String::from("activity-btn");
        if let Some(sig) = &active {
            if sig.get() {
                c.push_str(" active");
            }
        }
        c
    };

    let is_disabled = move || disabled.as_ref().map_or(false, |d| d.get());

    view! {
        <DelayedTooltip text=tip>
            <button
                class=btn_class
                aria-label=aria_label
                disabled=is_disabled
                on:click=move |_| on_click()
            >
                {children()}
            </button>
        </DelayedTooltip>
    }
}
