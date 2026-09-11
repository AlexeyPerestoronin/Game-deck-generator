//! Activity-bar control that runs `deck_gen::prepare_html` on the VFS.

use leptos::prelude::*;

use crate::ui::icons::PrepareHtmlIcon;
use crate::ui::tooltips::DelayedTooltip;
use crate::workspace::Workspace;

/// Generate HTML previews for every deck; errors go to `warning`.
#[component]
pub fn PrepareHtmlButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    view! {
        <DelayedTooltip text="Generate HTML preview for every deck in this workspace.">
            <button
                class="activity-btn"
                aria-label="prepare_html"
                disabled=move || workspace.loading.get()
                on:click=move |_| {
                    warning_title.set("Cannot prepare HTML".into());
                    workspace.prepare_html(warning);
                }
            >
                <PrepareHtmlIcon />
            </button>
        </DelayedTooltip>
    }
}
