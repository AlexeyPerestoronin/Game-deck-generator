//! Prompt + model selector for CreateGame / EditGame. Not a chat.

use leptos::prelude::*;

use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

/// Textarea + model `<select>` on a click-to-dismiss backdrop.
#[component]
pub fn AiPromptModal(
    open: RwSignal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] models: Signal<Vec<(String, String)>>,
    #[prop(into)] loading: Signal<bool>,
    #[prop(into)] on_run: Callback<(String, String)>,
) -> impl IntoView {
    let prompt = RwSignal::new(String::new());
    let model_path = RwSignal::new(String::new());

    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        prompt.set(String::new());
        let list = models.get();
        if !list.iter().any(|(p, _)| p == &model_path.get_untracked()) {
            model_path.set(list.first().map(|(p, _)| p.clone()).unwrap_or_default());
        }
    });

    let can_run = move || {
        !prompt.get().trim().is_empty()
            && !loading.get()
            && !models.get().is_empty()
            && !model_path.get().is_empty()
    };

    view! {
        <Show when=move || open.get()>
            <div
                class="modal-backdrop"
                role="presentation"
                on:click=move |_| open.set(false)
            >
                <div
                    class="modal modal-ai"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="ai-prompt-title"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <h2 id="ai-prompt-title" class="modal-title">{move || title.get()}</h2>
                    <label class="modal-field">
                        <span class="modal-field-label">{move || locale::localize(keys::MODAL_AI_MODEL)}</span>
                        <select
                            class="modal-select"
                            prop:value=move || model_path.get()
                            on:change=move |ev| model_path.set(event_target_value(&ev))
                        >
                            <For
                                each=move || models.get()
                                key=|(path, _)| path.clone()
                                children=move |(path, label)| {
                                    view! {
                                        <option value=path.clone()>{label}</option>
                                    }
                                }
                            />
                        </select>
                    </label>
                    <label class="modal-field">
                        <span class="modal-field-label">{move || locale::localize(keys::MODAL_AI_PROMPT)}</span>
                        <textarea
                            class="modal-textarea"
                            prop:value=move || prompt.get()
                            on:input=move |ev| prompt.set(event_target_value(&ev))
                        />
                    </label>
                    <div class="modal-actions">
                        <button class="modal-btn" on:click=move |_| open.set(false)>
                            {move || locale::localize(keys::MODAL_CANCEL)}
                        </button>
                        <button
                            class="modal-btn"
                            disabled=move || !can_run()
                            on:click=move |_| {
                                let p = prompt.get().trim().to_string();
                                let m = model_path.get();
                                if p.is_empty() || m.is_empty() {
                                    return;
                                }
                                open.set(false);
                                on_run.run((p, m));
                            }
                        >
                            {move || locale::localize(keys::MODAL_AI_RUN)}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
