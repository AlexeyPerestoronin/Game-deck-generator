//! Activity-bar toggle that splits the editor: files left, previews right.

use leptos::prelude::*;

use crate::buttons::ActivityButton;
use crate::icons::SplitPreviewIcon;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Split the working area so preview tabs occupy the right half.
#[component]
pub fn SplitPreviewButton(workspace: Workspace) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_SPLIT_PREVIEW).into_boxed_str());
    let aria = Signal::derive(move || locale::localize(keys::ARIA_SPLIT_PREVIEW));
    let active = Signal::derive(move || workspace.split_preview.get());

    view! {
        <ActivityButton
            tip=tip
            aria_label=aria
            active=active
            on_click=move || workspace.toggle_split_preview()
        >
            <SplitPreviewIcon />
        </ActivityButton>
    }
}
