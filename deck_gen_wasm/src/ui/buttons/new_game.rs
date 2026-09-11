//! Activity-bar control that installs the `new-game` template under `games/`.

use leptos::prelude::*;

use crate::ui::icons::NewGameIcon;
use crate::ui::tooltips::DelayedTooltip;
use crate::workspace::Workspace;

/// Fetch GitHub (or the local Trunk copy) and add a unique game folder.
#[component]
pub fn NewGameButton(workspace: Workspace) -> impl IntoView {
    view! {
        <DelayedTooltip text="Add a new game from the GitHub master template.">
            <button
                class="activity-btn"
                aria-label="New game"
                disabled=move || workspace.loading.get()
                on:click=move |_| workspace.add_new_game()
            >
                <NewGameIcon />
            </button>
        </DelayedTooltip>
    }
}
