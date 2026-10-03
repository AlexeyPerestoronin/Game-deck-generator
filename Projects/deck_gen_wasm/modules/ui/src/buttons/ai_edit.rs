//! Activity-bar control that opens the AI edit prompt for the current game file.

use leptos::prelude::*;

use crate::buttons::ActivityButton;
use crate::icons::AiEditIcon;
use crate::modals::AiPromptModal;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::{list_ai_models, AiRequest, Workspace};

/// Edit the active game source file with the AI agent.
#[component]
pub fn AiEditButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    let tip: &'static str = Box::leak(locale::localize(keys::TOOLTIP_AI_EDIT).into_boxed_str());
    let aria = Signal::derive(move || locale::localize(keys::ARIA_AI_EDIT));
    let show = RwSignal::new(false);
    let disabled = Signal::derive(move || {
        workspace.loading.get() || workspace.current_ai_edit_target().is_none()
    });

    view! {
        <ActivityButton
            tip=tip
            aria_label=aria
            disabled=disabled
            on_click=move || {
                if workspace.current_ai_edit_target().is_some() {
                    show.set(true);
                }
            }
        >
            <AiEditIcon />
        </ActivityButton>
        <AiPromptModal
            open=show
            title=Signal::derive(move || locale::localize(keys::MODAL_AI_EDIT_TITLE))
            models=Signal::derive(move || workspace.vfs.with(list_ai_models))
            loading=Signal::derive(move || workspace.loading.get())
            on_run=Callback::new(move |(prompt, model_path): (String, String)| {
                let Some((game, file)) = workspace.current_ai_edit_target() else {
                    return;
                };
                warning_title.set(locale::localize(keys::WARNING_CANNOT_RUN_AI));
                workspace.run_ai(
                    AiRequest::EditGame { game, file, prompt },
                    &model_path,
                    warning,
                );
            })
        />
    }
}
