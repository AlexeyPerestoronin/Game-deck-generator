//! Left icon strip: save, ZIP, new/load game, prepare HTML, prepare PDF, clear.
//!
//! Each action is its own button module. This bar owns the shared confirm/alert
//! dialogs and wires them to [`Workspace`](crate::workspace::Workspace).

use leptos::prelude::*;

use crate::ui::buttons::{
    ClearButton, DownloadButton, LoadGameButton, NewGameButton, PrepareHtmlButton,
    PreparePdfButton, SaveButton,
};
use crate::ui::modals::{AlertModal, ConfirmModal};
use crate::workspace::Workspace;

/// Vertical action bar bound to one [`Workspace`].
#[component]
pub fn ActivityBar(workspace: Workspace) -> impl IntoView {
    let show_clear = RwSignal::new(false);
    let show_load = RwSignal::new(false);
    let warning = RwSignal::new(None::<String>);
    let warning_title = RwSignal::new("Error".to_string());

    view! {
        <nav class="activity-bar" aria-label="Actions">
            <SaveButton workspace=workspace />
            <DownloadButton workspace=workspace />
            <NewGameButton workspace=workspace />
            <LoadGameButton workspace=workspace on_open=move |_| show_load.set(true) />
            <PrepareHtmlButton
                workspace=workspace
                warning=warning
                warning_title=warning_title
            />
            <PreparePdfButton
                workspace=workspace
                warning=warning
                warning_title=warning_title
            />
            <div class="activity-spacer"></div>
            <ClearButton on_open=move |_| show_clear.set(true) />
            <ConfirmModal
                open=show_clear
                title="Clear workspace?"
                message="This removes every file in the current browser session. It cannot be undone."
                confirm_label="Clear"
                danger=true
                on_cancel=move |_| show_clear.set(false)
                on_confirm=move |_| {
                    workspace.clear();
                    show_clear.set(false);
                }
            />
            <ConfirmModal
                open=show_load
                title="Load Game"
                message="Choose a folder on disk. The whole folder will be copied into games/. Only md, json, json5, html and scss files are allowed."
                confirm_label="Select folder"
                on_cancel=move |_| show_load.set(false)
                on_confirm=move |_| {
                    show_load.set(false);
                    warning_title.set("Cannot load folder".into());
                    workspace.load_game_from_disk(warning);
                }
            />
            <AlertModal
                open=Signal::derive(move || warning.get().is_some())
                title=Signal::derive(move || warning_title.get())
                message=Signal::derive(move || warning.get().unwrap_or_default())
                on_close=move |_| warning.set(None)
            />
        </nav>
    }
}
