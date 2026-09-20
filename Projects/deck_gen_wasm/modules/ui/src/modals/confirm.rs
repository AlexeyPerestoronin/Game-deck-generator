//! Yes/no overlay. `danger` styles the confirm button as destructive.

use leptos::prelude::*;

use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

/// Two-button dialog on a click-to-dismiss backdrop.
#[component]
pub fn ConfirmModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] message: Signal<String>,
    #[prop(into)] confirm_label: Signal<String>,
    #[prop(optional)] danger: bool,
    #[prop(into)] on_cancel: Callback<()>,
    #[prop(into)] on_confirm: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div
                class="modal-backdrop"
                role="presentation"
                on:click=move |_| on_cancel.run(())
            >
                <div
                    class="modal"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="confirm-title"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <h2 id="confirm-title" class="modal-title">{move || title.get()}</h2>
                    <p class="modal-body">{move || message.get()}</p>
                    <div class="modal-actions">
                        <button class="modal-btn" on:click=move |_| on_cancel.run(())>
                            {move || locale::localize(keys::MODAL_CANCEL)}
                        </button>
                        <button
                            class="modal-btn"
                            class:danger=danger
                            on:click=move |_| on_confirm.run(())
                        >
                            {move || confirm_label.get()}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
