//! Explorer header control that prompts for a new file name.

use leptos::prelude::*;

use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Create an empty file under the current explorer parent.
#[component]
pub fn NewFileButton(workspace: Workspace) -> impl IntoView {
    view! {
        <button class="text-btn" title=move || locale::localize(keys::EXPLORER_NEW_FILE) on:click=move |_| workspace.create_file()>
            {move || locale::localize(keys::BUTTON_NEW_FILE)}
        </button>
    }
}
