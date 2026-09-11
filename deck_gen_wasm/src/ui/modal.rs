//! Overlay dialogs used by the activity bar.
//!
//! [`ConfirmModal`] is yes/no (clear workspace, load-game warning).
//! [`AlertModal`] is a single OK for prepare-html / import errors. Both sit
//! on a click-to-dismiss backdrop; the dialog itself stops propagation.

use leptos::prelude::*;

/// Yes/no dialog. `danger` styles the confirm button as destructive.
#[component]
pub fn ConfirmModal(
    #[prop(into)] open: Signal<bool>,
    title: &'static str,
    message: &'static str,
    confirm_label: &'static str,
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
                    <h2 id="confirm-title" class="modal-title">{title}</h2>
                    <p class="modal-body">{message}</p>
                    <div class="modal-actions">
                        <button class="modal-btn" on:click=move |_| on_cancel.run(())>
                            "Cancel"
                        </button>
                        <button
                            class="modal-btn"
                            class:danger=danger
                            on:click=move |_| on_confirm.run(())
                        >
                            {confirm_label}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Single-button error/info dialog with reactive title and message.
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
