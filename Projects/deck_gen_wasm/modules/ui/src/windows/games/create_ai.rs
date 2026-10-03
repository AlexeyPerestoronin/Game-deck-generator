//! Local-games button: create a new game with one AI prompt.

use leptos::prelude::*;

use crate::modals::AiPromptModal;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::{list_ai_models, AiRequest, Workspace};

/// Full-width button under “Load Local Games”.
#[component]
pub fn CreateAiGameButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    let show = RwSignal::new(false);
    view! {
        <>
            <button
                class="games-full-btn"
                disabled=move || workspace.loading.get()
                on:click=move |_| show.set(true)
            >
                {move || locale::localize(keys::GAMES_CREATE_AI)}
            </button>
            <AiPromptModal
                open=show
                title=Signal::derive(move || locale::localize(keys::MODAL_AI_CREATE_TITLE))
                models=Signal::derive(move || workspace.vfs.with(list_ai_models))
                loading=Signal::derive(move || workspace.loading.get())
                on_run=Callback::new(move |(prompt, model_path): (String, String)| {
                    warning_title.set(locale::localize(keys::WARNING_CANNOT_RUN_AI));
                    workspace.run_ai(AiRequest::CreateGame { prompt }, &model_path, warning);
                })
            />
        </>
    }
}
