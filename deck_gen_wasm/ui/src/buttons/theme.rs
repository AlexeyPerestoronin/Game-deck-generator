//! Theme cycle button placed after Feedback in activity bar.
//! Three PNG states switched by .theme-dark / .theme-light / .theme-system class on the button.

use leptos::prelude::*;

use crate::icons::ThemeIcon;
use deck_gen_wasm_conf as conf;

#[component]
pub fn ThemeButton() -> impl IntoView {
    let current = RwSignal::new(crate::theme::load());

    let btn_class = move || {
        let t = current.get();
        format!("activity-btn theme-{}", t.as_str())
    };

    let aria = move || {
        match current.get() {
            crate::theme::ColorTheme::Dark => "Color theme: dark. Click to cycle.",
            crate::theme::ColorTheme::Light => "Color theme: light. Click to cycle.",
            crate::theme::ColorTheme::System => "Color theme: system. Click to cycle.",
        }
    };

    Effect::new({
        let sig = current;
        move |_| {
            // ensure signal matches persisted (e.g. after system init)
            sig.set(crate::theme::load());
        }
    });

    let on_click = move |_| {
        let next = crate::theme::cycle_and_apply();
        current.set(next);
    };

    view! {
        <div class="tooltip-host">
            <crate::tooltips::DelayedTooltip text=conf::ui::TOOLTIP_THEME>
                <button
                    class=btn_class
                    on:click=on_click
                    aria-label=aria
                >
                    <ThemeIcon />
                </button>
            </crate::tooltips::DelayedTooltip>
        </div>
    }
}
