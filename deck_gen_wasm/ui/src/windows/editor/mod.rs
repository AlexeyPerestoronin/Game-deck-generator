//! Open-file pane: tab strip plus source editor or HTML/Markdown/PDF/image preview.
//!
//! Tabs come from [`Workspace::tabs`](deck_gen_wasm_workspace::Workspace::tabs), or
//! from [`Workspace::preview_tabs`](deck_gen_wasm_workspace::Workspace::preview_tabs)
//! when split-for-preview is on. The focused textarea is the source of truth
//! while typing; VFS is updated after an idle pause. Markdown / JSON / HTML /
//! SCSS use syntect HTML behind a transparent textarea; everything else is a
//! plain `<textarea>`. Preview tabs render HTML in an iframe, Markdown as HTML,
//! PDF via a blob-URL iframe, and images via a blob-URL `<img>`.

use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

use deck_gen_wasm_conf::ui::SPLIT_PANE_MIN_WIDTH_PX;
use deck_gen_wasm_workspace::Workspace;

mod draft;
mod highlight;
mod highlighted;
mod iframe;
mod pane;
mod plain;
mod preview;
mod tab;

use pane::EditorPane;

/// Center pane: one editor, or two halves when split-for-preview is on.
#[component]
pub fn Editor(workspace: Workspace) -> impl IntoView {
    let split_ref = NodeRef::<Div>::new();
    let left_width = RwSignal::new(None::<u32>);
    let drag_active = RwSignal::new(false);
    let drag_start = RwSignal::new(None::<(i32, u32, u32)>);

    let left_style = move || {
        left_width
            .get()
            .map(|w| format!("{}px", w))
            .unwrap_or_else(|| "1fr".to_string())
    };

    // Attach window listeners once (on Editor creation) for drag (minimal duplicated logic).
    if let Some(win) = window() {
        let move_cb = {
            let left_width = left_width.clone();
            let drag_active = drag_active.clone();
            let drag_start = drag_start.clone();
            Closure::<dyn FnMut(_)>::new(move |ev: web_sys::MouseEvent| {
                if !drag_active.get() {
                    return;
                }
                if let Some((start_x, start_left, cont_w)) = drag_start.get() {
                    let dx = ev.client_x() - start_x;
                    let min_w = SPLIT_PANE_MIN_WIDTH_PX;
                    let mut new_left = (start_left as i32 + dx).max(min_w as i32) as u32;
                    let max_left = cont_w.saturating_sub(min_w + 4);
                    if new_left > max_left {
                        new_left = max_left;
                    }
                    left_width.set(Some(new_left));
                }
            })
        };
        let _ = win.add_event_listener_with_callback("mousemove", move_cb.as_ref().unchecked_ref());
        move_cb.forget();

        let up_cb = {
            let drag_active = drag_active.clone();
            let drag_start = drag_start.clone();
            Closure::<dyn FnMut(_)>::new(move |_ev: web_sys::MouseEvent| {
                if drag_active.get() {
                    drag_active.set(false);
                    drag_start.set(None);
                }
            })
        };
        let _ = win.add_event_listener_with_callback("mouseup", up_cb.as_ref().unchecked_ref());
        up_cb.forget();
    }

    view! {
        {move || {
            if workspace.split_preview.get() {
                view! {
                    <div class="editor-split" node_ref=split_ref style=("--left-pane-width", left_style)>
                        <EditorPane workspace=workspace preview_pane=false />
                        <div
                            class="resizer"
                            on:mousedown=move |ev: leptos::ev::MouseEvent| {
                                let Some(container) = split_ref.get() else { return; };
                                let cont_w = container.offset_width() as u32;
                                let min_total = 2 * SPLIT_PANE_MIN_WIDTH_PX + 4;
                                if cont_w < min_total {
                                    return;
                                }
                                let cur_left = left_width.get().unwrap_or(cont_w / 2);
                                drag_start.set(Some((ev.client_x(), cur_left, cont_w)));
                                drag_active.set(true);
                                ev.prevent_default();
                            }
                        ></div>
                        <EditorPane workspace=workspace preview_pane=true />
                    </div>
                }
                .into_any()
            } else {
                view! { <EditorPane workspace=workspace preview_pane=false /> }.into_any()
            }
        }}
    }
}
