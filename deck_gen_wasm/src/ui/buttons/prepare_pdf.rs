//! Activity-bar control that runs `deck_gen::prepare_pdf` in the browser.

use leptos::prelude::*;

use crate::ui::icons::PreparePdfIcon;
use crate::ui::tooltips::DelayedTooltip;
use crate::workspace::Workspace;

/// Generate card PDFs and A4 duplex sheets; errors go to `warning`.
#[component]
pub fn PreparePdfButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    view! {
        <DelayedTooltip text="Generate card PDFs and A4 duplex sheets for every deck in this workspace.">
            <button
                class="activity-btn"
                aria-label="prepare_pdf"
                disabled=move || workspace.loading.get()
                on:click=move |_| {
                    warning_title.set("Cannot prepare PDF".into());
                    workspace.prepare_pdf(warning);
                }
            >
                <PreparePdfIcon />
            </button>
        </DelayedTooltip>
    }
}
