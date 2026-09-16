//! Activity-bar control that runs `deck_gen::prepare_html` on the VFS.

use leptos::prelude::*;

use crate::icons::PrepareHtmlIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Generate HTML previews for every deck; errors go to `warning`.
#[component]
pub fn PrepareHtmlButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_PREPARE_HTML).into_boxed_str());
    view! {
        <DelayedTooltip text=tip>
            <button
                class="activity-btn"
                aria-label=move || locale::localize(keys::ARIA_PREPARE_HTML)
                disabled=move || workspace.loading.get()
                on:click=move |_| {
                    warning_title.set(locale::localize(keys::WARNING_CANNOT_PREPARE_HTML));
                    workspace.prepare_html(warning);
                }
            >
                <PrepareHtmlIcon />
            </button>
        </DelayedTooltip>
    }
}
