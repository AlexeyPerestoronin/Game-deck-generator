use leptos::prelude::*;

#[component]
pub fn ConfirmModal(
    #[prop(into)] open: Signal<bool>,
    title: &'static str,
    message: &'static str,
    confirm_label: &'static str,
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
                        <button class="modal-btn danger" on:click=move |_| on_confirm.run(())>
                            {confirm_label}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
