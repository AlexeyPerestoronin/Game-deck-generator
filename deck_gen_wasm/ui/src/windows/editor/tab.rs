//! One editor tab: label, activate on click, close, context menu (Close all), drag-to-reorder within pane.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::menus::{TabCloseMenu, TabContextMenu};
use deck_gen_wasm_fs::file_name;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::{OpenTab, TabKind, Workspace};

/// Tab chip. `preview_pane` compares active state against the right-hand strip.
#[component]
pub(super) fn EditorTab(workspace: Workspace, tab: OpenTab, preview_pane: bool) -> impl IntoView {
    let tab_for_active = tab.clone();
    let tab_for_click = tab.clone();
    let tab_for_close = tab.clone();
    let tab_for_dnd = tab.clone(); // for dragstart data
    let tab_for_this = tab.clone(); // for this tab as hovered in its dragover
    let path = tab.path.clone();
    let kind = tab.kind;

    let (menu_pos, set_menu_pos) = signal::<Option<(f64, f64)>>(None);

    let show_menu = move || {
        menu_pos.get().map(|(x, y)| {
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
        })
    };

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
            on:dragstart=move |ev| {
                let d_ev = ev.unchecked_ref::<web_sys::DragEvent>();
                if let Some(dt) = d_ev.data_transfer() {
                    let kind_str = if preview_pane { "preview" } else { "edit" };
                    let data = format!("{}|{}", kind_str, tab_for_dnd.path);
                    let _ = dt.set_data("text/plain", &data);
                    let _ = dt.set_effect_allowed("move");
                }
            }
            on:dragover=move |ev| {
                ev.prevent_default();
                let d_ev = ev.unchecked_ref::<web_sys::DragEvent>();
                let dt = match d_ev.data_transfer() {
                    Some(d) => d,
                    None => return,
                };
                dt.set_drop_effect("move");
                let data = match dt.get_data("text/plain") {
                    Ok(d) => d,
                    Err(_) => return,
                };
                let mut parts = data.splitn(2, '|');
                let kind_str = parts.next().unwrap_or("");
                let path = parts.next().unwrap_or("").to_string();
                if path.is_empty() { return; }
                let dragged = OpenTab {
                    path,
                    kind: if kind_str == "preview" { TabKind::Preview } else { TabKind::Edit },
                };
                let source_is_preview = dragged.kind == TabKind::Preview;
                if source_is_preview != preview_pane {
                    return;
                }
                let hovered = tab_for_this.clone();
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
            }
            on:dragend=move |_ev| {
                // optional cleanup, data is in browser
            }
        >
            <span class="editor-tab-label">
                {move || match kind {
                    TabKind::Edit => file_name(&path).to_string(),
                    TabKind::Preview => format!(
                        "{}{}",
                        locale::localize(keys::TAB_PREVIEW_PREFIX),
                        file_name(&path)
                    ),
                }}
            </span>
            <button
                class="editor-tab-close"
                type="button"
                title=move || locale::localize(keys::TAB_CLOSE)
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
