//! Games activity button.
//! Toggles SidebarMode::Games (Local/Global sections). Re-click hides. Active style when open.

use leptos::prelude::*;

use crate::icons::GamesIcon;
use crate::sidebar::SidebarMode;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

/// Toggles sidebar to Games. Same hide-on-reclick rule as Explorer.
#[component]
pub fn GamesButton(
    sidebar_mode: RwSignal<SidebarMode>,
    explorer_width: RwSignal<i32>,
    last_explorer_width: RwSignal<i32>,
) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_GAMES).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                class:active=move || sidebar_mode.get() == SidebarMode::Games
                aria-label=move || locale::localize(keys::ARIA_GAMES)
                on:click=move |_| {
                    let cur = sidebar_mode.get_untracked();
                    let next = crate::sidebar::toggle_sidebar(cur, SidebarMode::Games);
                    sidebar_mode.set(next);
                    if next == SidebarMode::Hidden {
                        explorer_width.set(0);
                    } else if explorer_width.get_untracked() <= 0 {
                        let w = last_explorer_width.get_untracked().max(260);
                        explorer_width.set(w);
                    }
                }
            >
                <GamesIcon />
            </button>
        </DelayedTooltip>
    }
}
