//! One editor tab: label, activate on click, close, context menu (Close all), drag-to-reorder within pane.

use leptos::prelude::*;
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
    let tab_for_dragover = tab.clone();
    let label = match tab.kind {
        TabKind::Edit => file_name(&tab.path).to_string(),
        TabKind::Preview => format!("Preview {}", file_name(&tab.path)),
    };

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

    // StoredValues for native drag-and-drop based reorder (avoids manual listeners which didn't fire reliably).
    // We use our own storage instead of dataTransfer to avoid web_sys feature requirements.
    let drag_tab = StoredValue::new(None::<OpenTab>);
    let drag_source_is_preview = StoredValue::new(false);

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
            draggable="true"
            on:click=move |_| workspace.activate_tab(tab_for_click.clone())
            on:contextmenu=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                set_menu_pos.set(Some((ev.client_x() as f64, ev.client_y() as f64)));
            }
            on:dragstart=move |_ev| {
                drag_tab.set_value(Some(tab_for_dnd.clone()));
                drag_source_is_preview.set_value(preview_pane);
            }
            on:dragover=move |ev| {
                ev.prevent_default();
                // Perform live reorder here: as the ghost is dragged over this tab,
                // move the dragged item in the list so tabs visually jump to new positions.
                // This way, even if drop event doesn't "commit", the position gets fixed live.
                let Some(dragged) = drag_tab.get_value() else { return; };
                let source_is_preview = drag_source_is_preview.get_value();
                if source_is_preview != preview_pane {
                    return;
                }
                let hovered = tab_for_dragover.clone();
                let list = if preview_pane {
                    workspace.preview_tabs.get()
                } else {
                    workspace.tabs.get()
                };
                if let Some(h_idx) = list.iter().position(|t| t == &hovered) {
                    let client_x = ev.client_x() as f64;
                    let before = if let Some(ct) = ev.current_target() {
                        if let Ok(el) = ct.dyn_into::<web_sys::Element>() {
                            let r = el.get_bounding_client_rect();
                            client_x < r.left() + r.width() / 2.0
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    let to_idx = if before { h_idx } else { h_idx + 1 };
                    workspace.move_tab(dragged, to_idx);
                }
            }
            on:drop=move |ev| {
                ev.prevent_default();
                // The actual reordering happens live in dragover as the ghost moves over tabs.
                // Drop just confirms end of gesture (cleanup happens in dragend too).
                let _ = drag_tab.get_value(); // touch to keep any prior state if needed
                drag_tab.set_value(None);
                drag_source_is_preview.set_value(false);
            }
            on:dragend=move |_| {
                drag_tab.set_value(None);
                drag_source_is_preview.set_value(false);
            }
        >
            <span class="editor-tab-label">{label}</span>
            <button
                class="editor-tab-close"
                type="button"
                title="Close"
                draggable="false"
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
