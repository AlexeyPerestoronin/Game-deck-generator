//! Activity-bar control that installs the `new-game` template under `games/`.

use leptos::prelude::*;

use crate::ui::icons::NewGameIcon;
use crate::ui::tooltips::DelayedTooltip;
use crate::workspace::Workspace;

/// Fetch GitHub and add a unique game folder; errors go to `warning`.
#[component]
pub fn NewGameButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    view! {
        <DelayedTooltip text="Add a new game from the GitHub master template.">
            <button
                class="activity-btn"
                aria-label="New game"
                disabled=move || workspace.loading.get()
                on:click=move |_| {
                    warning_title.set("Cannot load new-game".into());
                    workspace.add_new_game(warning);
                }
            >
                <NewGameIcon />
            </button>
        </DelayedTooltip>
    }
}
