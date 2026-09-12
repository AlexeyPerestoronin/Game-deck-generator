//! Activity-bar control that opens the load-folder confirm dialog.

use leptos::prelude::*;

use crate::ui::icons::LoadGameIcon;
use crate::ui::tooltips::DelayedTooltip;
use crate::workspace::Workspace;

/// Ask the user to confirm, then pick a disk folder into `games/`.
#[component]
pub fn LoadGameButton(workspace: Workspace, #[prop(into)] on_open: Callback<()>) -> impl IntoView {
    view! {
        <DelayedTooltip text="Load a game folder from disk into games/.">
            <button
                class="activity-btn"
                aria-label="Load Game"
                disabled=move || workspace.loading.get()
                on:click=move |_| on_open.run(())
            >
                <LoadGameIcon />
            </button>
        </DelayedTooltip>
    }
}
