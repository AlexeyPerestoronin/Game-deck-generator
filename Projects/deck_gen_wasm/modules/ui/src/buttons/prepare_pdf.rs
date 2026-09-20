//! Activity-bar control that runs `deck_gen::prepare_pdf` in the browser.

use leptos::prelude::*;

use crate::buttons::ActivityButton;
use crate::icons::PreparePdfIcon;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Generate card PDFs and A4 duplex sheets; errors go to `warning`.
#[component]
pub fn PreparePdfButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_PREPARE_PDF).into_boxed_str());
    let aria = Signal::derive(move || locale::localize(keys::ARIA_PREPARE_PDF));
    let disabled = Signal::derive(move || workspace.loading.get());

    view! {
        <ActivityButton
            tip=tip
            aria_label=aria
            disabled=disabled
            on_click=move || {
                warning_title.set(locale::localize(keys::WARNING_CANNOT_PREPARE_PDF));
                workspace.prepare_pdf(warning);
            }
        >
            <PreparePdfIcon />
        </ActivityButton>
    }
}
