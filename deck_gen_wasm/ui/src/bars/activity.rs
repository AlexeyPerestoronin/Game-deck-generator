//! Left icon strip: ZIP, new/load game, prepare HTML/PDF, split preview, clear.
//!
//! Each action is its own button module. This bar owns the shared confirm/alert
//! dialogs and wires them to [`Workspace`](deck_gen_wasm_workspace::Workspace).

use leptos::prelude::*;

use crate::buttons::{
    ClearButton, DownloadButton, FeedbackButton, LoadGameButton, LocaleButton, NewGameButton,
    PrepareHtmlButton, PreparePdfButton, SplitPreviewButton, ThemeButton,
};
use crate::modals::{AlertModal, ConfirmModal};
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Vertical action bar bound to one [`Workspace`].
#[component]
pub fn ActivityBar(workspace: Workspace) -> impl IntoView {
    let show_clear = RwSignal::new(false);
    let show_load = RwSignal::new(false);
    let warning = RwSignal::new(None::<String>);
    let warning_title = RwSignal::new(locale::localize(keys::WARNING_ERROR));

    view! {
        <nav class="activity-bar" aria-label="Actions">
            <DownloadButton workspace=workspace />
            <NewGameButton
                workspace=workspace
                warning=warning
                warning_title=warning_title
            />
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
            <SplitPreviewButton workspace=workspace />
            <div class="activity-spacer"></div>
            <ClearButton on_open=move |_| show_clear.set(true) />
            <FeedbackButton />
            <ThemeButton />
            <LocaleButton />
            <ConfirmModal
                open=show_clear
                title=Signal::derive(move || locale::localize(keys::CONFIRM_CLEAR_TITLE))
                message=Signal::derive(move || locale::localize(keys::CONFIRM_CLEAR_MESSAGE))
                confirm_label=Signal::derive(move || locale::localize(keys::CONFIRM_CLEAR_LABEL))
                danger=true
                on_cancel=move |_| show_clear.set(false)
                on_confirm=move |_| {
                    workspace.clear();
                    show_clear.set(false);
                }
            />
            <ConfirmModal
                open=show_load
                title=Signal::derive(move || locale::localize(keys::CONFIRM_LOAD_TITLE))
                message=Signal::derive(move || locale::localize(keys::CONFIRM_LOAD_MESSAGE))
                confirm_label=Signal::derive(move || locale::localize(keys::CONFIRM_LOAD_LABEL))
                on_cancel=move |_| show_load.set(false)
                on_confirm=move |_| {
                    show_load.set(false);
                    warning_title.set(locale::localize(keys::WARNING_CANNOT_LOAD_FOLDER));
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
