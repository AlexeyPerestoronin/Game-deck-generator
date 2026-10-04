//! GamesPanel: SidebarMode::Games content (Local/Global sections like VSCode Extensions).
//!
//! Collapsible sections default to 50/50 height split of the sidebar. Vertical drag resizer
//! between them adjusts proportions. Local lists persist-VFS games; Global lists Deck-Games.

mod create_ai;

use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent as WasmMouseEvent;

use create_ai::CreateAiGameButton;
use deck_gen_wasm_browser as js;
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::kind;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_template::{info_matches_query, CatalogEntry, GameInfo, DEFAULT_ICON_PNG};
use deck_gen_wasm_workspace::{game_card_name, local_games, TabKind, Workspace};

/// Returns CSS `flex` values for Local/Global sections depending on collapse flags and split ratio.
fn section_flex(local_collapsed: bool, global_collapsed: bool, ratio: f32) -> (String, String) {
    if local_collapsed {
        (
            "0 0 auto".to_string(),
            if global_collapsed {
                "0 0 auto".to_string()
            } else {
                "1 1 0".to_string()
            },
        )
    } else if global_collapsed {
        (format!("{} 1 0", ratio), "0 0 auto".to_string())
    } else {
        (format!("{} 1 0", ratio), format!("{} 1 0", 1.0 - ratio))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MenuState {
    id: String,
    left: f64,
    top: f64,
    local: bool,
}

fn preview_path_of(root: &str, info: &GameInfo) -> Option<String> {
    let name = info.preview.as_ref().filter(|s| !s.is_empty())?;
    if root.is_empty() {
        Some(format!("{}/{}", conf::catalog::PREVIEW_DIR, name))
    } else {
        Some(format!("{root}/{}/{}", conf::catalog::PREVIEW_DIR, name))
    }
}

fn icon_path_of(root: &str, info: &GameInfo) -> Option<String> {
    let name = info.icon.as_ref().filter(|s| !s.is_empty())?;
    if root.is_empty() {
        Some(format!("{}/{}", conf::catalog::PREVIEW_DIR, name))
    } else {
        Some(format!("{root}/{}/{}", conf::catalog::PREVIEW_DIR, name))
    }
}

fn fallback_preview_path(root: &str) -> String {
    format!("__defaults__/{root}/preview.html")
}

fn row_is_selected(workspace: Workspace, root: &str, info: &GameInfo) -> bool {
    let path = if workspace.split_preview.get() {
        workspace.active_preview_tab.get().map(|tab| tab.path)
    } else {
        workspace
            .active_tab
            .get()
            .filter(|tab| tab.kind == TabKind::Preview)
            .map(|tab| tab.path)
    };
    let Some(path) = path else {
        return false;
    };
    if let Some(preview) = preview_path_of(root, info) {
        if path == preview {
            return true;
        }
    }
    path == fallback_preview_path(root)
}

#[component]
pub fn GamesPanel(
    workspace: Workspace,
    show_load: RwSignal<bool>,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    let local_collapsed = RwSignal::new(false);
    let global_collapsed = RwSignal::new(false);
    let split_ratio = RwSignal::new(0.5f32);
    let game_menu = RwSignal::new(None::<MenuState>);
    let search_input = RwSignal::new(String::new());
    let applied_query = RwSignal::new(String::new());

    let is_dragging = RwSignal::new(false);
    let drag_start_y = RwSignal::new(0i32);
    let drag_start_ratio = RwSignal::new(0.5f32);
    let container_ref: NodeRef<html::Div> = NodeRef::new();

    Effect::new(move |_| {
        workspace.ensure_catalog();
    });

    Effect::new({
        let ratio = split_ratio;
        let dragging = is_dragging;
        let sy = drag_start_y;
        let sr = drag_start_ratio;
        let cref = container_ref;
        move |_| {
            let win = web_sys::window().expect("window");
            let mm = Closure::<dyn FnMut(WasmMouseEvent)>::new(move |ev: WasmMouseEvent| {
                if !dragging.get_untracked() {
                    return;
                }
                let dy = ev.client_y() - sy.get_untracked();
                let h = cref
                    .get()
                    .map(|c| c.offset_height() as f32)
                    .unwrap_or(300.0)
                    .max(50.0);
                let r = (sr.get_untracked() + (dy as f32) / h).clamp(0.1, 0.9);
                ratio.set(r);
            });
            let mu = Closure::<dyn FnMut(WasmMouseEvent)>::new(move |_ev: WasmMouseEvent| {
                dragging.set(false);
            });
            let _ = win.add_event_listener_with_callback("mousemove", mm.as_ref().unchecked_ref());
            let _ = win.add_event_listener_with_callback("mouseup", mu.as_ref().unchecked_ref());
            mm.forget();
            mu.forget();
        }
    });

    let start_vdrag = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();
        is_dragging.set(true);
        drag_start_y.set(ev.client_y());
        drag_start_ratio.set(split_ratio.get_untracked());
    };

    let toggle_local = move |_| local_collapsed.update(|b| *b = !*b);
    let toggle_global = move |_| global_collapsed.update(|b| *b = !*b);

    let trigger_load_local = move |_| {
        show_load.set(true);
    };

    let open_game_settings = move |id: String, local: bool, ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        let left = ev.client_x() as f64 + 4.0;
        let top = ev.client_y() as f64;
        game_menu.set(Some(MenuState {
            id,
            left,
            top,
            local,
        }));
    };

    let close_game_menu = move || {
        game_menu.set(None);
    };

    let apply_search = move |_| {
        applied_query.set(search_input.get());
    };

    let ratio = split_ratio;
    let lc = local_collapsed;
    let gc = global_collapsed;
    let local_flex = move || {
        let (l, _) = section_flex(lc.get(), gc.get(), ratio.get());
        l
    };
    let global_flex = move || {
        let (_, g) = section_flex(lc.get(), gc.get(), ratio.get());
        g
    };

    let chevron = |collapsed: bool| if collapsed { "▸" } else { "▾" };

    view! {
        <aside class="explorer games">
            <div class="games-container" node_ref=container_ref>
                <div class="games-section" style=move || format!("flex: {};", local_flex())>
                    <div class="games-section-header" on:click=toggle_local>
                        <span class="chevron">{move || chevron(local_collapsed.get())}</span>
                        <span>{move || locale::localize(keys::GAMES_LOCAL)}</span>
                    </div>
                    <Show when=move || !local_collapsed.get()>
                        <div class="games-section-body">
                            <button
                                class="games-full-btn"
                                disabled=move || workspace.loading.get()
                                on:click=trigger_load_local
                            >
                                {move || locale::localize(keys::GAMES_LOAD_LOCAL)}
                            </button>
                            <CreateAiGameButton
                                workspace=workspace
                                warning=warning
                                warning_title=warning_title
                            />
                            <For
                                each=move || workspace.vfs.with(|vfs| local_games(vfs))
                                key=|g| g.root.clone()
                                children=move |game| {
                                    let root = game.root.clone();
                                    let info = game.info.clone();
                                    let root_open = root.clone();
                                    let info_open = info.clone();
                                    let root_sel = root.clone();
                                    let info_sel = info.clone();
                                    let root_menu = root.clone();
                                    view! {
                                        <GameCard
                                            workspace=workspace
                                            root=root
                                            info=info
                                            from_temp=false
                                            selected=Signal::derive(move || row_is_selected(workspace, &root_sel, &info_sel))
                                            on_open=move |_| workspace.open_game_preview(&root_open, &info_open, false)
                                            on_settings=move |ev| open_game_settings(root_menu.clone(), true, ev)
                                        />
                                    }
                                }
                            />
                        </div>
                    </Show>
                </div>

                <Show when=move || !local_collapsed.get() && !global_collapsed.get()>
                    <div class="games-hresizer" on:mousedown=start_vdrag></div>
                </Show>

                <div class="games-section" style=move || format!("flex: {};", global_flex())>
                    <div class="games-section-header" on:click=toggle_global>
                        <span class="chevron">{move || chevron(global_collapsed.get())}</span>
                        <span>{move || locale::localize(keys::GAMES_GLOBAL)}</span>
                    </div>
                    <Show when=move || !global_collapsed.get()>
                        <div class="games-section-body games-global-body">
                            <div class="games-search">
                                <input
                                    type="text"
                                    class="games-search-input"
                                    placeholder=move || locale::localize(keys::GAMES_SEARCH_PLACEHOLDER)
                                    prop:value=move || search_input.get()
                                    on:input=move |ev| search_input.set(event_target_value(&ev))
                                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                        if ev.key() == "Enter" {
                                            applied_query.set(search_input.get());
                                        }
                                    }
                                />
                                <button class="games-search-btn" on:click=apply_search>
                                    {move || locale::localize(keys::GAMES_SEARCH)}
                                </button>
                            </div>
                            <Show when=move || workspace.catalog_loading.get()>
                                <div class="games-status">{move || locale::localize(keys::GAMES_LOADING)}</div>
                            </Show>
                            <Show when=move || workspace.catalog_error.get().is_some()>
                                <div class="games-status error">{move || workspace.catalog_error.get().unwrap_or_default()}</div>
                            </Show>
                            <For
                                each=move || {
                                    let q = applied_query.get();
                                    workspace
                                        .catalog_games
                                        .get()
                                        .into_iter()
                                        .filter(|g| info_matches_query(&g.info, &q))
                                        .collect::<Vec<_>>()
                                }
                                key=|g: &CatalogEntry| g.root.clone()
                                children=move |game: CatalogEntry| {
                                    let root = game.root.clone();
                                    let info = game.info.clone();
                                    let root_open = root.clone();
                                    let info_open = info.clone();
                                    let root_sel = root.clone();
                                    let info_sel = info.clone();
                                    let root_menu = root.clone();
                                    view! {
                                        <GameCard
                                            workspace=workspace
                                            root=root
                                            info=info
                                            from_temp=true
                                            selected=Signal::derive(move || row_is_selected(workspace, &root_sel, &info_sel))
                                            on_open=move |_| workspace.open_game_preview(&root_open, &info_open, true)
                                            on_settings=move |ev| open_game_settings(root_menu.clone(), false, ev)
                                        />
                                    }
                                }
                            />
                            <Show when=move || {
                                !workspace.catalog_loading.get()
                                    && workspace.catalog_error.get().is_none()
                                    && workspace.catalog_games.get().is_empty()
                            }>
                                <div class="games-status">{move || locale::localize(keys::GAMES_EMPTY)}</div>
                            </Show>
                            <Show when=move || {
                                !workspace.catalog_loading.get()
                                    && workspace.catalog_error.get().is_none()
                                    && !workspace.catalog_games.get().is_empty()
                                    && {
                                        let q = applied_query.get();
                                        workspace
                                            .catalog_games
                                            .get()
                                            .iter()
                                            .all(|g| !info_matches_query(&g.info, &q))
                                    }
                            }>
                                <div class="games-status">{move || locale::localize(keys::GAMES_SEARCH_EMPTY)}</div>
                            </Show>
                        </div>
                    </Show>
                </div>
            </div>

            <Show when=move || game_menu.get().is_some()>
                {move || {
                    if let Some(m) = game_menu.get() {
                        let id = m.id.clone();
                        let local = m.local;
                        view! {
                            <div class="context-backdrop" on:click=move |_| close_game_menu()>
                                <div
                                    class="context-menu"
                                    role="menu"
                                    style=format!("left:{}px;top:{}px", m.left, m.top)
                                    on:click=move |ev| ev.stop_propagation()
                                >
                                    {if local {
                                        let id_html = id.clone();
                                        let id_pdf = id.clone();
                                        view! {
                                            <>
                                                <button
                                                    class="context-item"
                                                    role="menuitem"
                                                    disabled=move || workspace.loading.get()
                                                    on:click=move |_| {
                                                        close_game_menu();
                                                        warning_title.set(locale::localize(keys::WARNING_CANNOT_PREPARE_HTML));
                                                        workspace.prepare_html_for(&[id_html.clone()], warning);
                                                    }
                                                >
                                                    {move || locale::localize(keys::ARIA_PREPARE_HTML)}
                                                </button>
                                                <button
                                                    class="context-item"
                                                    role="menuitem"
                                                    disabled=move || workspace.loading.get()
                                                    on:click=move |_| {
                                                        close_game_menu();
                                                        warning_title.set(locale::localize(keys::WARNING_CANNOT_PREPARE_PDF));
                                                        workspace.prepare_pdf_for(&[id_pdf.clone()], warning);
                                                    }
                                                >
                                                    {move || locale::localize(keys::ARIA_PREPARE_PDF)}
                                                </button>
                                            </>
                                        }.into_any()
                                    } else {
                                        let id_load = id.clone();
                                        view! {
                                            <button
                                                class="context-item"
                                                role="menuitem"
                                                disabled=move || workspace.loading.get()
                                                on:click=move |_| {
                                                    close_game_menu();
                                                    warning_title.set(locale::localize(keys::WARNING_CANNOT_LOAD_NEW_GAME));
                                                    workspace.add_game_from_catalog(&id_load, warning);
                                                }
                                            >
                                                {move || locale::localize(keys::GAMES_LOAD)}
                                            </button>
                                        }.into_any()
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }
                }}
            </Show>
        </aside>
    }
}

#[component]
fn GameCard<F, S>(
    workspace: Workspace,
    root: String,
    info: GameInfo,
    from_temp: bool,
    selected: Signal<bool>,
    on_open: F,
    on_settings: S,
) -> impl IntoView
where
    F: Fn(leptos::ev::MouseEvent) + Clone + 'static + Send,
    S: Fn(leptos::ev::MouseEvent) + Clone + 'static + Send,
{
    let icon_path = icon_path_of(&root, &info);
    let name_info = info.clone();
    view! {
        <div
            class=move || {
                if selected.get() {
                    "game-row selected"
                } else {
                    "game-row"
                }
            }
            on:click=on_open
        >
            <GameIcon workspace=workspace path=icon_path from_temp=from_temp />
            <span class="game-name">{move || game_card_name(&name_info)}</span>
            <button
                class="game-settings-btn"
                on:click=move |ev| {
                    ev.stop_propagation();
                    on_settings(ev);
                }
            >
                {move || locale::localize(keys::GAMES_SETTINGS)}
            </button>
        </div>
    }
}

#[component]
fn GameIcon(workspace: Workspace, path: Option<String>, from_temp: bool) -> impl IntoView {
    let src = RwSignal::new(String::new());
    Effect::new(move |_| {
        let (bytes, mime) = match &path {
            Some(p) => {
                let data = if from_temp {
                    workspace
                        .temp_vfs
                        .with(|vfs| vfs.read_bytes(p).map(Vec::from))
                } else {
                    workspace.vfs.with(|vfs| vfs.read_bytes(p).map(Vec::from))
                };
                match data {
                    Some(bytes) => (bytes, kind::image_mime(p)),
                    None => (DEFAULT_ICON_PNG.to_vec(), "image/png"),
                }
            }
            None => (DEFAULT_ICON_PNG.to_vec(), "image/png"),
        };
        let next = js::blob_url(&bytes, mime).unwrap_or_default();
        let prev = src.get_untracked();
        src.set(next.clone());
        if prev != next {
            js::revoke_object_url(&prev);
        }
    });
    on_cleanup(move || js::revoke_object_url(&src.get_untracked()));
    view! {
        <img class="game-icon" prop:src=move || src.get() alt="" />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ratio_gives_equal_flex() {
        let (l, g) = section_flex(false, false, 0.5);
        assert!(l.starts_with("0.5"));
        assert!(g.starts_with("0.5"));
    }

    #[test]
    fn collapsed_local_gives_full_to_global() {
        let (l, g) = section_flex(true, false, 0.3);
        assert_eq!(l, "0 0 auto");
        assert_eq!(g, "1 1 0");
    }

    #[test]
    fn both_collapsed_gives_auto_stack_at_top() {
        let (l, g) = section_flex(true, true, 0.7);
        assert_eq!(l, "0 0 auto");
        assert_eq!(g, "0 0 auto");
    }

    #[test]
    fn ratio_30_70() {
        let (l, g) = section_flex(false, false, 0.3);
        assert!(l.starts_with("0.3"));
        assert!(g.starts_with("0.7"));
    }
}
