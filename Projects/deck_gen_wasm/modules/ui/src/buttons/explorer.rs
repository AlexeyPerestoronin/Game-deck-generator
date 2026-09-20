//! Explorer activity button.
//! Toggles SidebarMode::Explorer (shows file tree). Re-click hides (width=0). Uses active style + 4-frame icon.

use leptos::prelude::*;

use crate::icons::ExplorerIcon;
use crate::sidebar::SidebarMode;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

/// Toggles sidebar to Explorer. When already Explorer, hides it (width -> 0).
/// When activating and width is 0, caller side restores previous non-zero width.
#[component]
pub fn ExplorerButton(
    sidebar_mode: RwSignal<SidebarMode>,
    explorer_width: RwSignal<i32>,
    last_explorer_width: RwSignal<i32>,
) -> impl IntoView {
    let tip: &'static str =
        Box::leak(locale::localize(keys::TOOLTIP_EXPLORER).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                class:active=move || sidebar_mode.get() == SidebarMode::Explorer
                aria-label=move || locale::localize(keys::ARIA_EXPLORER)
                on:click=move |_| {
                    let cur = sidebar_mode.get_untracked();
                    let next = crate::sidebar::toggle_sidebar(cur, SidebarMode::Explorer);
                    sidebar_mode.set(next);
                    if next == SidebarMode::Hidden {
                        explorer_width.set(0);
                    } else if explorer_width.get_untracked() <= 0 {
                        let w = last_explorer_width.get_untracked().max(260);
                        explorer_width.set(w);
                    }
                }
            >
                <ExplorerIcon />
            </button>
        </DelayedTooltip>
    }
}
