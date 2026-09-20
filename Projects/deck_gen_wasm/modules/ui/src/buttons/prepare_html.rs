//! Activity-bar control that runs `deck_gen::prepare_html` on the VFS.

use leptos::prelude::*;

use crate::buttons::ActivityButton;
use crate::icons::PrepareHtmlIcon;
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
    let aria = Signal::derive(move || locale::localize(keys::ARIA_PREPARE_HTML));
    let disabled = Signal::derive(move || workspace.loading.get());

    view! {
        <ActivityButton
            tip=tip
            aria_label=aria
            disabled=disabled
            on_click=move || {
                warning_title.set(locale::localize(keys::WARNING_CANNOT_PREPARE_HTML));
                workspace.prepare_html(warning);
            }
        >
            <PrepareHtmlIcon />
        </ActivityButton>
    }
}
