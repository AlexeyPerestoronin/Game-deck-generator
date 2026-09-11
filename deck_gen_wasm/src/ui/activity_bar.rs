//! Left icon strip: save, ZIP, new/load game, prepare HTML, prepare PDF, clear.
//!
//! Each button is a delayed tooltip plus a click that calls into
//! [`Workspace`](crate::workspace::Workspace). Destructive / warning flows
//! (clear, load-game confirm, prepare-html errors) open the shared modals.

use leptos::prelude::*;

use super::icons::{
    ClearIcon, DownloadIcon, LoadGameIcon, NewGameIcon, PrepareHtmlIcon, PreparePdfIcon, SaveIcon,
};
use super::modal::{AlertModal, ConfirmModal};
use super::tooltip::DelayedTooltip;
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
            <DelayedTooltip text="Save the workspace in this browser. A page reload will restore it.">
                <button
                    class="activity-btn"
                    aria-label="Save"
                    on:click=move |_| workspace.persist()
                >
                    <SaveIcon />
                </button>
            </DelayedTooltip>
            <DelayedTooltip text="Download the workspace as a ZIP archive.">
                <button
                    class="activity-btn"
                    aria-label="Download"
                    on:click=move |_| workspace.download()
                >
                    <DownloadIcon />
                </button>
            </DelayedTooltip>
            <DelayedTooltip text="Add a new game from the GitHub master template.">
                <button
                    class="activity-btn"
                    aria-label="New game"
                    disabled=move || workspace.loading.get()
                    on:click=move |_| workspace.add_new_game()
                >
                    <NewGameIcon />
                </button>
            </DelayedTooltip>
            <DelayedTooltip text="Load a game folder from disk into games/.">
                <button
                    class="activity-btn"
                    aria-label="Load Game"
                    disabled=move || workspace.loading.get()
                    on:click=move |_| show_load.set(true)
                >
                    <LoadGameIcon />
                </button>
            </DelayedTooltip>
            <DelayedTooltip text="Generate HTML preview for every deck in this workspace.">
                <button
                    class="activity-btn"
                    aria-label="prepare_html"
                    disabled=move || workspace.loading.get()
                    on:click=move |_| {
                        warning_title.set("Cannot prepare HTML".into());
                        workspace.prepare_html(warning);
                    }
                >
                    <PrepareHtmlIcon />
                </button>
            </DelayedTooltip>
            <DelayedTooltip text="Generate card PDFs and A4 duplex sheets for every deck in this workspace.">
                <button
                    class="activity-btn"
                    aria-label="prepare_pdf"
                    disabled=move || workspace.loading.get()
                    on:click=move |_| {
                        warning_title.set("Cannot prepare PDF".into());
                        workspace.prepare_pdf(warning);
                    }
                >
                    <PreparePdfIcon />
                </button>
            </DelayedTooltip>
            <div class="activity-spacer"></div>
            <DelayedTooltip text="Clear the workspace in this browser.">
                <button
                    class="activity-btn"
                    aria-label="Clear"
                    on:click=move |_| show_clear.set(true)
                >
                    <ClearIcon />
                </button>
            </DelayedTooltip>
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
