//! Locale cycle button (EN/RU) placed under Theme in activity bar.
//! Two-state icon switched by current locale; class on button for potential styling.

use leptos::prelude::*;

use crate::icons::LocaleIcon;
use deck_gen_wasm_locale::{self as locale, keys};

#[component]
pub fn LocaleButton() -> impl IntoView {
    let current = RwSignal::new(locale::get_active_locale());

    let btn_class = move || {
        let s = match current.get() {
            locale::Locale::En => "en",
            locale::Locale::Ru => "ru",
        };
        format!("activity-btn locale-{}", s)
    };

    let aria = move || locale::localize(keys::ARIA_LOCALE);

    Effect::new({
        let sig = current;
        move |_| {
            sig.set(locale::get_active_locale());
        }
    });

    let on_click = move |_| {
        let _ = locale::change_locale();
        current.set(locale::get_active_locale());
    };

    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_LOCALE).into_boxed_str());
    view! {
        <div class="tooltip-host">
            <crate::tooltips::DelayedTooltip text=tip>
                <button
                    class=btn_class
                    on:click=on_click
                    aria-label=aria
                >
                    <LocaleIcon />
                </button>
            </crate::tooltips::DelayedTooltip>
        </div>
    }
}
