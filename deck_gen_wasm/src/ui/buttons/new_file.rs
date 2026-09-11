//! Explorer header control that prompts for a new file name.

use leptos::prelude::*;

use crate::workspace::Workspace;

/// Create an empty file under the current explorer parent.
#[component]
pub fn NewFileButton(workspace: Workspace) -> impl IntoView {
    view! {
        <button class="text-btn" title="New File" on:click=move |_| workspace.create_file()>
            "+ File"
        </button>
    }
}
