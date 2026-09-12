//! File tree for the in-memory workspace.
//!
//! Rows are a flattened walk of expanded directories. Click selects (and
//! toggles folders); right-click opens the explorer context menu (including
//! folder “load file(s)”). Create-file / create-folder buttons live in the
//! header; status is the footer. Import errors from load-files use an alert.

use leptos::prelude::*;

use crate::ui::buttons::{NewFileButton, NewFolderButton};
use crate::ui::menus::{ChosenCommand, ContextMenu, MenuState};
use crate::ui::modals::AlertModal;
use crate::workspace::Workspace;

mod tree;

use tree::TreeRows;

/// Sidebar tree bound to one [`Workspace`].
#[component]
pub fn Explorer(workspace: Workspace) -> impl IntoView {
    let menu = RwSignal::new(None::<MenuState>);
    let warning = RwSignal::new(None::<String>);

    view! {
        <aside class="explorer">
            <div class="explorer-title-row">
                <span class="explorer-title">"Games"</span>
            </div>
            <header class="explorer-header">
                <div class="explorer-actions">
                    <NewFileButton workspace=workspace />
                    <NewFolderButton workspace=workspace />
                </div>
            </header>
            <div class="tree" role="tree" on:click=move |_| menu.set(None)>
                <TreeRows workspace=workspace menu=menu />
            </div>
            <footer class="status">{move || workspace.status.get()}</footer>
            <Show when=move || menu.get().is_some()>
                {move || menu.get().map(|state| {
                    view! {
                        <ContextMenu
                            state=state
                            on_dismiss=move |_| menu.set(None)
                            on_command=move |chosen: ChosenCommand| {
                                workspace.run_entry_command(chosen.id, &chosen.path, warning);
                            }
                        />
                    }
                })}
            </Show>
            <AlertModal
                open=Signal::derive(move || warning.get().is_some())
                title=Signal::derive(move || "Cannot load files".to_string())
                message=Signal::derive(move || warning.get().unwrap_or_default())
                on_close=move |_| warning.set(None)
            />
        </aside>
    }
}
