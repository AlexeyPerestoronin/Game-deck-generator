use leptos::prelude::*;

use super::icons::{DownloadIcon, SaveIcon};
use super::tooltip::DelayedTooltip;
use crate::workspace::Workspace;

#[component]
pub fn ActivityBar(workspace: Workspace) -> impl IntoView {
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
        </nav>
    }
}
