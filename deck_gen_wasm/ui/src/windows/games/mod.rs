//! GamesPanel: SidebarMode::Games content (Local/Global sections like VSCode Extensions).
//!
//! Collapsible sections default to 50/50 height split of the sidebar. Vertical drag resizer
//! between them adjusts proportions. Local has load button; Global lists GitHub games.


use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent as WasmMouseEvent;

use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::Workspace;

/// Maps collapse state and ratio into (local, global) flex factors for the sections.
fn section_flex(local_collapsed: bool, global_collapsed: bool, ratio: f32) -> (f64, f64) {
    if local_collapsed {
        (0.0, if global_collapsed { 0.0 } else { 1.0 })
    } else if global_collapsed {
        (1.0, 0.0)
    } else {
        (ratio as f64, (1.0 - ratio) as f64)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
struct MenuState {
    game: String,
    left: f64,
    top: f64,
}

#[component]
pub fn GamesPanel(
    workspace: Workspace,
    show_load: RwSignal<bool>,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
) -> impl IntoView {
    // Collapsible headers (false = expanded, default per spec).
    let local_collapsed = RwSignal::new(false);
    let global_collapsed = RwSignal::new(false);

    // Height split ratio for bodies (local fraction). 0.5 = equal.
    let split_ratio = RwSignal::new(0.5f32);

    // Global list (fetched once on mount of this panel).
    let games = RwSignal::new(Vec::<String>::new());
    let loading = RwSignal::new(false);
    let fetch_error = RwSignal::new(None::<String>);

    // Per-game settings menu (simple, anchored).
    let game_menu = RwSignal::new(None::<MenuState>);

    // Drag state for vertical split between Local/Global sections.
    let is_dragging = RwSignal::new(false);
    let drag_start_y = RwSignal::new(0i32);
    let drag_start_ratio = RwSignal::new(0.5f32);
    let container_ref: NodeRef<html::Div> = NodeRef::new();

    // One-time fetch on first render of Games panel.
    Effect::new({
        let g = games;
        let l = loading;
        let e = fetch_error;
        move |_| {
            if !g.get_untracked().is_empty() || l.get_untracked() {
                return;
            }
            l.set(true);
            e.set(None);
            spawn_local(async move {
                match deck_gen_wasm_template::list_game_folders().await {
                    Ok(list) => {
                        g.set(list);
                    }
                    Err(err) => {
                        e.set(Some(err));
                    }
                }
                l.set(false);
            });
        }
    });

    // Global listeners for vertical resizer (closures leaked).
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
                let h = cref.get().map(|c| c.offset_height() as f32).unwrap_or(300.0).max(50.0);
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
        if let Some(c) = container_ref.get() {
            // We don't strictly need height here; heuristic works for ratio.
            is_dragging.set(true);
            drag_start_y.set(ev.client_y());
            drag_start_ratio.set(split_ratio.get_untracked());
        }
    };

    let toggle_local = move |_| local_collapsed.update(|b| *b = !*b);
    let toggle_global = move |_| global_collapsed.update(|b| *b = !*b);

    let trigger_load_local = move |_| {
        show_load.set(true);
    };

    let open_game_settings = move |game: String, ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        // Anchor near click for the small menu (reuse context style).
        let left = ev.client_x() as f64 + 4.0;
        let top = ev.client_y() as f64;
        game_menu.set(Some(MenuState { game, left, top }));
    };

    let close_game_menu = move || {
        game_menu.set(None);
    };

    let load_selected_game = move |game: String| {
        close_game_menu();
        warning_title.set(locale::localize(keys::WARNING_CANNOT_LOAD_NEW_GAME));
        workspace.add_game_from_github(&game, warning);
    };

    // Section flex via pure helper. 0.5/0.5 => equal split of full height.
    let local_flex = move || {
        let (l, _) = section_flex(local_collapsed.get(), global_collapsed.get(), split_ratio.get());
        l
    };
    let global_flex = move || {
        let (_, g) = section_flex(local_collapsed.get(), global_collapsed.get(), split_ratio.get());
        g
    };

    let chevron = |collapsed: bool| if collapsed { "▸" } else { "▾" };

    view! {
        <aside class="explorer games">
            <div class="games-container" node_ref=container_ref>
                // Local
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
                        </div>
                    </Show>
                </div>

                // Vertical resizer between sections (only when both expanded)
                <Show when=move || !local_collapsed.get() && !global_collapsed.get()>
                    <div class="games-hresizer" on:mousedown=start_vdrag></div>
                </Show>

                // Global
                <div class="games-section" style=move || format!("flex: {};", global_flex())>
                    <div class="games-section-header" on:click=toggle_global>
                        <span class="chevron">{move || chevron(global_collapsed.get())}</span>
                        <span>{move || locale::localize(keys::GAMES_GLOBAL)}</span>
                    </div>
                    <Show when=move || !global_collapsed.get()>
                        <div class="games-section-body games-global-body">
                            <Show when=move || loading.get()>
                                <div class="games-status">...</div>
                            </Show>
                            <Show when=move || fetch_error.get().is_some()>
                                <div class="games-status error">{move || fetch_error.get().unwrap_or_default()}</div>
                            </Show>
                            <For
                                each=move || games.get()
                                key=|g| g.clone()
                                children=move |g| {
                                    let g_for_click = g.clone();
                                    let g_for_label = g.clone();
                                    view! {
                                        <div class="game-row">
                                            <span class="game-name">{g_for_label}</span>
                                            <button
                                                class="game-settings-btn"
                                                on:click=move |ev| open_game_settings(g_for_click.clone(), ev)
                                            >
                                                {move || locale::localize(keys::GAMES_SETTINGS)}
                                            </button>
                                        </div>
                                    }
                                }
                            />
                            <Show when=move || !loading.get() && games.get().is_empty() && fetch_error.get().is_none()>
                                <div class="games-status">(no games listed)</div>
                            </Show>
                        </div>
                    </Show>
                </div>
            </div>

            // Per-game popup menu (one item: Load)
            <Show when=move || game_menu.get().is_some()>
                {move || {
                    if let Some(m) = game_menu.get() {
                        let game = m.game.clone();
                        view! {
                            <div class="context-backdrop" on:click=move |_| close_game_menu()>
                                <div
                                    class="context-menu"
                                    role="menu"
                                    style=format!("left:{}px;top:{}px", m.left, m.top)
                                    on:click=move |ev| ev.stop_propagation()
                                >
                                    <button
                                        class="context-item"
                                        role="menuitem"
                                        on:click=move |_| load_selected_game(game.clone())
                                    >
                                        {move || locale::localize(keys::GAMES_LOAD)}
                                    </button>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ratio_gives_equal_flex() {
        let (l, g) = section_flex(false, false, 0.5);
        assert!((l - 0.5).abs() < 1e-6);
        assert!((g - 0.5).abs() < 1e-6);
    }

    #[test]
    fn collapsed_local_gives_full_to_global() {
        let (l, g) = section_flex(true, false, 0.3);
        assert_eq!(l, 0.0);
        assert_eq!(g, 1.0);
    }

    #[test]
    fn both_collapsed_gives_zero() {
        let (l, g) = section_flex(true, true, 0.7);
        assert_eq!(l, 0.0);
        assert_eq!(g, 0.0);
    }

    #[test]
    fn ratio_30_70() {
        let (l, g) = section_flex(false, false, 0.3);
        assert!((l - 0.3).abs() < 1e-6);
        assert!((g - 0.7).abs() < 1e-6);
    }
}
