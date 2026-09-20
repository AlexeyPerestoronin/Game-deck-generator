//! Explorer header control that prompts for a new folder name.

use leptos::prelude::*;

use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Create a folder under the current explorer parent.
#[component]
pub fn NewFolderButton(workspace: Workspace) -> impl IntoView {
    view! {
        <button class="text-btn" title=move || locale::localize(keys::EXPLORER_NEW_FOLDER) on:click=move |_| workspace.create_folder()>
            {move || locale::localize(keys::BUTTON_NEW_FOLDER)}
        </button>
    }
}
