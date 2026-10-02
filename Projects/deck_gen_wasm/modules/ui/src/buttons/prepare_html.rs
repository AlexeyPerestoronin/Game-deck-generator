//! Activity-bar control that runs `deck_gen::prepare_html` on selected games.

use leptos::prelude::*;

use crate::buttons::prepare_menu::{PrepareKind, PrepareScopedButton};
use deck_gen_wasm_workspace::Workspace;

/// Generate HTML previews for selected local games; errors go to `warning`.
#[component]
pub fn PrepareHtmlButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    view! {
        <PrepareScopedButton
            workspace=workspace
            warning=warning
            warning_title=warning_title
            kind=PrepareKind::Html
        />
    }
}
