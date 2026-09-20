//! Activity-bar control that installs the `new-game` template under `games/`.

use leptos::prelude::*;

use crate::icons::NewGameIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Fetch GitHub and add a unique game folder; errors go to `warning`.
#[component]
pub fn NewGameButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_NEW_GAME).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                aria-label=move || locale::localize(keys::ARIA_NEW_GAME)
                disabled=move || workspace.loading.get()
                on:click=move |_| {
                    warning_title.set(locale::localize(keys::WARNING_CANNOT_LOAD_NEW_GAME));
                    workspace.add_new_game(warning);
                }
            >
                <NewGameIcon />
            </button>
        </DelayedTooltip>
    }
}
