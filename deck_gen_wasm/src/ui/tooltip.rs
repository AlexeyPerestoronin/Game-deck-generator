//! Tooltip that appears after the pointer stays on the control for 1.5s.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

const HOVER_DELAY_MS: u32 = 1500;

#[component]
pub fn DelayedTooltip(text: &'static str, children: Children) -> impl IntoView {
    let visible = RwSignal::new(false);
    let generation = RwSignal::new(0u32);

    let on_enter = move |_| {
        visible.set(false);
        let token = generation.get_untracked().wrapping_add(1);
        generation.set(token);
        spawn_local(async move {
            TimeoutFuture::new(HOVER_DELAY_MS).await;
            if generation.get_untracked() == token {
                visible.set(true);
            }
        });
    };
    let on_leave = move |_| {
        generation.update(|token| *token = token.wrapping_add(1));
        visible.set(false);
    };

    view! {
        <div class="tooltip-host" on:mouseenter=on_enter on:mouseleave=on_leave>
            {children()}
            <Show when=move || visible.get()>
                <div class="tooltip" role="tooltip">{text}</div>
            </Show>
        </div>
    }
}
