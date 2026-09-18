//! ActivityBar: the left vertical strip of icon buttons (VSCode activity bar style).
//!
//! Top group controls sidebar mode (Explorer/Games) or actions that leave sidebar unchanged.
//! Bottom: Clear + Settings (menu).
//! Owns the shared Confirm/Alert modals used by Clear and by Load (from Local section too).
//! Receives sidebar signals from LoadedApp so buttons can coordinate width+mode.

use leptos::prelude::*;

use crate::buttons::{
    ClearButton, DownloadButton, ExplorerButton, GamesButton, PrepareHtmlButton, PreparePdfButton,
    SettingsButton, SplitPreviewButton,
};
use crate::modals::{AlertModal, ConfirmModal};
use crate::sidebar::SidebarMode;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Vertical action bar bound to one [`Workspace`].
#[component]
pub fn ActivityBar(
    workspace: Workspace,
    sidebar_mode: RwSignal<SidebarMode>,
    explorer_width: RwSignal<i32>,
    last_explorer_width: RwSignal<i32>,
    // Lifted so Games panel (and old Load button if any) can trigger the same confirm.
    show_load: RwSignal<bool>,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    let show_clear = RwSignal::new(false);

    view! {
        <nav class="activity-bar" aria-label="Actions">
            <ExplorerButton
                sidebar_mode=sidebar_mode
                explorer_width=explorer_width
                last_explorer_width=last_explorer_width
            />
            <GamesButton
                sidebar_mode=sidebar_mode
                explorer_width=explorer_width
                last_explorer_width=last_explorer_width
            />
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
            <DownloadButton workspace=workspace />
            <SplitPreviewButton workspace=workspace />
            <div class="activity-spacer"></div>
            <ClearButton on_open=move |_| show_clear.set(true) />
            <SettingsButton />
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
