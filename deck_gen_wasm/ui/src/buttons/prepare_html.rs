//! Activity-bar control that runs `deck_gen::prepare_html` on the VFS.

use leptos::prelude::*;

use crate::icons::PrepareHtmlIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_workspace::Workspace;

/// Generate HTML previews for every deck; errors go to `warning`.
#[component]
pub fn PrepareHtmlButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    view! {
        <DelayedTooltip text=conf::ui::TOOLTIP_PREPARE_HTML>
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
