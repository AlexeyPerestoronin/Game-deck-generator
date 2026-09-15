//! Tooltip that appears after the pointer stays on the control.
//!
//! Uses a generation counter to cancel pending timers. The tip uses a NodeRef +
//! getBoundingClientRect + position:fixed so it can appear next to activity-bar
//! buttons without being clipped by overflow:hidden ancestors.
//! Delay is [`deck_gen_wasm_conf::ui::TOOLTIP_HOVER_DELAY_MS`].

use gloo_timers::future::TimeoutFuture;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use deck_gen_wasm_conf as conf;

/// Wrap `children` with a delayed hover tooltip showing `text`.
#[component]
pub fn DelayedTooltip(text: &'static str, children: Children) -> impl IntoView {
    let visible = RwSignal::new(false);
    let generation = RwSignal::new(0u32);
    let host: NodeRef<html::Div> = NodeRef::new();
    let pos = RwSignal::new((0.0f64, 0.0f64));

    let on_enter = move |_| {
        visible.set(false);
        let token = generation.get_untracked().wrapping_add(1);
        generation.set(token);

        // Capture viewport position so the tooltip can be placed with
        // position:fixed. This lets it escape overflow:hidden on .activity-bar
        // and .ide (the tooltip appears to the right of the narrow bar).
        if let Some(el) = host.get() {
            let rect = el.get_bounding_client_rect();
            let left = rect.right() + 10.0;
            let top = rect.top() + rect.height() / 2.0;
            pos.set((left, top));
        }

        spawn_local(async move {
            TimeoutFuture::new(conf::ui::TOOLTIP_HOVER_DELAY_MS).await;
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
        <div class="tooltip-host" node_ref=host on:mouseenter=on_enter on:mouseleave=on_leave>
            {children()}
            <Show when=move || visible.get()>
                <div
                    class="tooltip"
                    style=move || {
                        let (left, top) = pos.get();
                        format!(
                            "position: fixed; left: {}px; top: {}px; transform: translateY(-50%);",
                            left, top
                        )
                    }
                    role="tooltip"
                >
                    {text}
                </div>
            </Show>
        </div>
    }
}
