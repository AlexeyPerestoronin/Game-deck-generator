//! Explorer header control that prompts for a new folder name.

use leptos::prelude::*;

use crate::workspace::Workspace;

/// Create a folder under the current explorer parent.
#[component]
pub fn NewFolderButton(workspace: Workspace) -> impl IntoView {
    view! {
        <button class="text-btn" title="New Folder" on:click=move |_| workspace.create_folder()>
            "+ Folder"
        </button>
    }
}
