//! One editor tab: label, activate on click, close, context menu (Close all), drag-to-reorder within pane.

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use deck_gen_wasm_fs::file_name;
use deck_gen_wasm_workspace::{OpenTab, TabKind, Workspace};
use crate::menus::{TabCloseMenu, TabContextMenu};

/// Tab chip. `preview_pane` compares active state against the right-hand strip.
#[component]
pub(super) fn EditorTab(workspace: Workspace, tab: OpenTab, preview_pane: bool) -> impl IntoView {
    let tab_for_active = tab.clone();
    let tab_for_click = tab.clone();
    let tab_for_close = tab.clone();
    let tab_for_dnd = tab.clone();
    let label = match tab.kind {
        TabKind::Edit => file_name(&tab.path).to_string(),
        TabKind::Preview => format!("Preview {}", file_name(&tab.path)),
    };

    // Attributes for drag hit-testing (cloned early so not moved by view closures).
    let attr_path = tab.path.clone();
    let attr_kind = match tab.kind {
        TabKind::Edit => "edit",
        TabKind::Preview => "preview",
    };
    let attr_pane = if preview_pane { "preview" } else { "edit" };

    let (menu_pos, set_menu_pos) = signal::<Option<(f64, f64)>>(None);

    let show_menu = move || menu_pos.get().map(|(x, y)| {
        view! {
            <TabContextMenu
                state=TabCloseMenu { x, y }
                on_dismiss=move |_| set_menu_pos.set(None)
                on_close_all=move |_| {
                    set_menu_pos.set(None);
                    workspace.close_all_tabs();
                }
            />
        }
    });

    // StoredValue holds the tab being dragged (set on mousedown). Mousemove closures
    // capture the originating pane bool locally. Guarded so leftover listeners are cheap no-ops.
    let drag_tab = StoredValue::new(None::<OpenTab>);

    view! {
        <div
            class="editor-tab"
            class:active=move || {
                let active = if preview_pane {
                    workspace.active_preview_tab.get()
                } else {
                    workspace.active_tab.get()
                };
                active.as_ref() == Some(&tab_for_active)
            }
            role="tab"
            title=tab.path.clone()
            attr:data-path=attr_path
            attr:data-kind=attr_kind
            attr:data-pane=attr_pane
            on:click=move |_| workspace.activate_tab(tab_for_click.clone())
            on:contextmenu=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                set_menu_pos.set(Some((ev.client_x() as f64, ev.client_y() as f64)));
            }
            on:mousedown=move |ev| {
                // Do not drag when starting on the close button.
                if let Some(t) = ev.target() {
                    if let Some(el) = t.dyn_ref::<web_sys::Element>() {
                        if el.class_list().contains("editor-tab-close") {
                            return;
                        }
                    }
                }
                if ev.button() != 0 {
                    return;
                }
                ev.prevent_default();
                drag_tab.set_value(Some(tab_for_dnd.clone()));
                let dragged = tab_for_dnd.clone();
                let from_preview = preview_pane;
                let ws = workspace;
                // Attach window listeners (forget for minimal; guarded by stored value).
                if let Some(win) = web_sys::window() {
                    // mousemove: find tab under pointer via elementFromPoint + data attrs, compute target idx, move live.
                    let move_cl = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |mev: web_sys::MouseEvent| {
                        let mx = mev.client_x() as f64;
                        let my = mev.client_y() as f64;
                        if drag_tab.get_value().is_none() {
                            return;
                        }
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            if let Some(el) = doc.element_from_point(mx as f32, my as f32) {
                                let mut cur = Some(el);
                                while let Some(node) = cur {
                                    if node.class_list().contains("editor-tab") {
                                        if let (Some(p), Some(k), Some(pn)) = (
                                            node.get_attribute("data-path"),
                                            node.get_attribute("data-kind"),
                                            node.get_attribute("data-pane"),
                                        ) {
                                            if pn == (if from_preview { "preview" } else { "edit" }) {
                                                let hovered = OpenTab {
                                                    path: p,
                                                    kind: if k == "preview" { TabKind::Preview } else { TabKind::Edit },
                                                };
                                                let list = if from_preview {
                                                    ws.preview_tabs.get()
                                                } else {
                                                    ws.tabs.get()
                                                };
                                                if let Some(h_idx) = list.iter().position(|t| t == &hovered) {
                                                    let rect = node.get_bounding_client_rect();
                                                    let before = mx < rect.left() + rect.width() / 2.0;
                                                    let to_idx = if before { h_idx } else { h_idx + 1 };
                                                    ws.move_tab(dragged.clone(), to_idx);
                                                }
                                            }
                                        }
                                        break;
                                    }
                                    cur = node.parent_element();
                                }
                            }
                        }
                    });
                    let _ = win.add_event_listener_with_callback("mousemove", move_cl.as_ref().unchecked_ref());
                    move_cl.forget();

                    // mouseup: end this drag session (listeners stay but guarded).
                    let up_cl = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_mev: web_sys::MouseEvent| {
                        drag_tab.set_value(None);
                    });
                    let _ = win.add_event_listener_with_callback("mouseup", up_cl.as_ref().unchecked_ref());
                    up_cl.forget();
                }
            }
        >
            <span class="editor-tab-label">{label}</span>
            <button
                class="editor-tab-close"
                type="button"
                title="Close"
                on:mousedown=move |ev| { ev.stop_propagation(); }
                on:click=move |ev| {
                    ev.stop_propagation();
                    workspace.close_tab(tab_for_close.clone());
                }
            >
                "×"
            </button>
        </div>

        {show_menu}
    }
}
