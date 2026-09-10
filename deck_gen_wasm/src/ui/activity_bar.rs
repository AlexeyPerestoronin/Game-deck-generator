use leptos::prelude::*;

use super::icons::{ClearIcon, DownloadIcon, NewGameIcon, SaveIcon};
use super::modal::ConfirmModal;
use super::tooltip::DelayedTooltip;
use crate::workspace::Workspace;

#[component]
pub fn ActivityBar(workspace: Workspace) -> impl IntoView {
    let show_clear = RwSignal::new(false);

    view! {
        <nav class="activity-bar" aria-label="Actions">
            <DelayedTooltip text="Save the workspace in this browser. A page reload will restore it.">
                <button
                    class="activity-btn"
                    aria-label="Save"
                    on:click=move |_| workspace.persist()
                >
                    <SaveIcon />
                </button>
            </DelayedTooltip>
            <DelayedTooltip text="Download the workspace as a ZIP archive.">
                <button
                    class="activity-btn"
                    aria-label="Download"
                    on:click=move |_| workspace.download()
                >
                    <DownloadIcon />
                </button>
            </DelayedTooltip>
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
            <div class="activity-spacer"></div>
            <DelayedTooltip text="Clear the workspace in this browser.">
                <button
                    class="activity-btn"
                    aria-label="Clear"
                    on:click=move |_| show_clear.set(true)
                >
                    <ClearIcon />
                </button>
            </DelayedTooltip>
            <ConfirmModal
                open=show_clear
                title="Clear workspace?"
                message="This removes every file in the current browser session. It cannot be undone."
                confirm_label="Clear"
                on_cancel=move |_| show_clear.set(false)
                on_confirm=move |_| {
                    workspace.clear();
                    show_clear.set(false);
                }
            />
        </nav>
    }
}
