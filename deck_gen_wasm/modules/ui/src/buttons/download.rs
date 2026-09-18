//! Activity-bar control that encodes the VFS as a ZIP download.

use leptos::prelude::*;

use crate::buttons::ActivityButton;
use crate::icons::DownloadIcon;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Offer the workspace as `workspace.zip`.
#[component]
pub fn DownloadButton(workspace: Workspace) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_DOWNLOAD).into_boxed_str());
    let aria = Signal::derive(move || locale::localize(keys::ARIA_DOWNLOAD));

    view! {
        <ActivityButton
            tip=tip
            aria_label=aria
            on_click=move || workspace.download()
        >
            <DownloadIcon />
        </ActivityButton>
    }
}
