//! Single-button error/info overlay with reactive title and message.

use leptos::prelude::*;

/// OK dialog on a click-to-dismiss backdrop.
#[component]
pub fn AlertModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] message: Signal<String>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || open.get()>
            <div
                class="modal-backdrop"
                role="presentation"
                on:click=move |_| on_close.run(())
            >
                <div
                    class="modal"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="alert-title"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <h2 id="alert-title" class="modal-title">{move || title.get()}</h2>
                    <p class="modal-body modal-pre">{move || message.get()}</p>
                    <div class="modal-actions">
                        <button class="modal-btn" on:click=move |_| on_close.run(())>
                            "OK"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
