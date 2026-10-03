//! Prompt + model selector for CreateGame / EditGame. Not a chat.

use leptos::prelude::*;

use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::ModelFile;

/// Textarea + model `<select>` on a click-to-dismiss backdrop.
#[component]
pub fn AiPromptModal(
    open: RwSignal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] models: Signal<Vec<ModelFile>>,
    #[prop(into)] loading: Signal<bool>,
    #[prop(into)] on_run: Callback<(String, String, String)>,
) -> impl IntoView {
    let prompt = RwSignal::new(String::new());
    let model_path = RwSignal::new(String::new());
    let api_key = RwSignal::new(String::new());
    let show_key = RwSignal::new(false);

    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        prompt.set(String::new());
        show_key.set(false);
        let list = models.get();
        if !list.iter().any(|m| m.path == model_path.get_untracked()) {
            model_path.set(list.first().map(|m| m.path.clone()).unwrap_or_default());
        }
    });

    let can_run = move || {
        !prompt.get().trim().is_empty()
            && !loading.get()
            && !models.get().is_empty()
            && !model_path.get().is_empty()
    };

    let hosting_url = Memo::new(move |_| {
        let path = model_path.get();
        models
            .get()
            .into_iter()
            .find(|m| m.path == path)
            .and_then(|m| m.api_key_hosting)
    });

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
                    <div class="modal-field">
                        <span class="modal-field-label">{move || locale::localize(keys::MODAL_AI_MODEL)}</span>
                        <div class="modal-ai-model-row">
                            <select
                                class="modal-select"
                                prop:value=move || model_path.get()
                                on:change=move |ev| model_path.set(event_target_value(&ev))
                            >
                                <For
                                    each=move || models.get()
                                    key=|m| m.path.clone()
                                    children=move |m| {
                                        let path = m.path.clone();
                                        view! {
                                            <option value=path>{m.label}</option>
                                        }
                                    }
                                />
                            </select>
                            <Show when=move || hosting_url.get().is_some()>
                                <button
                                    type="button"
                                    class="modal-btn modal-ai-hosting"
                                    on:click=move |_| {
                                        let Some(url) = hosting_url.get() else {
                                            return;
                                        };
                                        if let Some(w) = web_sys::window() {
                                            let _ = w.open_with_url_and_target(&url, "_blank");
                                        }
                                    }
                                >
                                    {move || locale::localize(keys::MODAL_AI_API_KEY_REQUEST)}
                                </button>
                            </Show>
                        </div>
                    </div>
                    <label class="modal-field">
                        <span class="modal-field-label">{move || locale::localize(keys::MODAL_AI_PROMPT)}</span>
                        <textarea
                            class="modal-textarea"
                            prop:value=move || prompt.get()
                            on:input=move |ev| prompt.set(event_target_value(&ev))
                        />
                    </label>
                    <div class="modal-actions">
                        <div class="modal-ai-key">
                            <input
                                class="modal-ai-key-input"
                                type=move || if show_key.get() { "text" } else { "password" }
                                prop:value=move || api_key.get()
                                on:input=move |ev| api_key.set(event_target_value(&ev))
                                placeholder=move || locale::localize(keys::MODAL_AI_API_KEY)
                                aria-label=move || locale::localize(keys::MODAL_AI_API_KEY)
                                autocomplete="off"
                                spellcheck="false"
                            />
                            <button
                                type="button"
                                class="modal-ai-key-toggle"
                                aria-label=move || {
                                    if show_key.get() {
                                        locale::localize(keys::MODAL_AI_API_KEY_HIDE)
                                    } else {
                                        locale::localize(keys::MODAL_AI_API_KEY_SHOW)
                                    }
                                }
                                on:click=move |_| show_key.update(|v| *v = !*v)
                            >
                                <Show
                                    when=move || show_key.get()
                                    fallback=|| view! { <EyeIcon /> }
                                >
                                    <EyeOffIcon />
                                </Show>
                            </button>
                        </div>
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
                                let key = api_key.get();
                                open.set(false);
                                on_run.run((p, m, key));
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

/// Open eye: click to reveal the API key.
#[component]
fn EyeIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M8 3C3.8 3 1.2 6.2 1 8c.2 1.8 2.8 5 7 5s6.8-3.2 7-5c-.2-1.8-2.8-5-7-5zm0 8.2A3.2 3.2 0 1 1 8 4.8a3.2 3.2 0 0 1 0 6.4z"
            />
            <circle fill="currentColor" cx="8" cy="8" r="1.6"/>
        </svg>
    }
}

/// Crossed-out eye: click to hide the API key.
#[component]
fn EyeOffIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M2.1 1.4 1.4 2.1 4 4.7C2.4 5.8 1.3 7.3 1 8c.3 1.8 2.8 5 7 5 1.4 0 2.6-.4 3.6-1l2.3 2.3.7-.7zM8 11.2c-1.8 0-3.2-1.4-3.2-3.2 0-.5.1-1 .4-1.4l4.2 4.2c-.4.3-.9.4-1.4.4zm7-3.2c-.2-1.1-1.1-2.5-2.4-3.5L14.6 2.5l-.7-.7-12 12 .7.7 2.1-2.1C5.8 12.7 6.9 13 8 13c4.2 0 6.8-3.2 7-5zM8 4.8c.5 0 1 .1 1.4.4L7.1 7.5A3.2 3.2 0 0 1 8 4.8z"
            />
        </svg>
    }
}
