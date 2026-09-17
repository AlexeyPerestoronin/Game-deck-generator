//! Root Leptos view: activity bar, explorer, and editor over one [`Workspace`].
//!
//! Session restore waits for IndexedDB binaries (PDF / images) so they are
//! present before the tree renders. Text state comes from localStorage. If
//! [`deck_gen_wasm_conf::help::PATH`] is missing from the VFS the bundled help file
//! is copied to the workspace root; if no editor tab is open, that file is
//! shown as a preview. An effect writes the text snapshot back to localStorage
//! whenever VFS/selection/expanded change (debounced after the editor flush),
//! and rewrites IndexedDB only when the binary fingerprint changes.

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent as WasmMouseEvent;

use crate::bars::{ActivityBar, ProgressRay};
use crate::sidebar::SidebarMode;
use crate::theme;
use crate::windows::{Editor, Explorer, GamesPanel};
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_persist::{
    binaries_fingerprint, encode_binaries, load_binaries, load_session, save_encoded, save_session,
};
use deck_gen_wasm_workspace::Workspace;

/// Shell layout: load persisted state, then the three panes.
#[component]
pub fn App() -> impl IntoView {
    let workspace_slot = RwSignal::new(None::<Workspace>);
    spawn_local(async move {
        let mut session = load_session().unwrap_or_default();
        if let Ok(entries) = load_binaries().await {
            for (path, data) in entries {
                let _ = session.vfs.put_bytes(&path, data);
            }
        }
        let workspace = Workspace::from_session(session);
        workspace.ensure_user_help();
        workspace_slot.set(Some(workspace));
    });

    // Snapshot loading text at App setup time (uses current persisted locale via localize).
    // Loading is transient; with hard-reload on locale switch the new language is picked on restart.
    // Captured String keeps child node structurally identical to a literal.
    let loading_text = locale::localize(keys::EDITOR_LOADING);

    view! {
        {move || match workspace_slot.get() {
            None => view! {
                <div class="ide">
                    <div class="editor-empty">{loading_text.clone()}</div>
                </div>
            }
            .into_any(),
            Some(workspace) => view! { <LoadedApp workspace=workspace /> }.into_any(),
        }}
    }
}

/// Autosave + three panes once localStorage and IndexedDB have been merged.
#[component]
fn LoadedApp(workspace: Workspace) -> impl IntoView {
    // Apply persisted (or default System) theme once on mount. Also sets up OS listener for System.
    Effect::new(|_| { theme::apply(); });

    // Locale: ensure reactive signal (inside owner) and set <html lang> + document.title from persisted (default EN).
    Effect::new(|_| {
        locale::ensure_locale_signal();
        locale::apply_initial_document();
    });

    let last_fp = RwSignal::new(workspace.vfs.with_untracked(binaries_fingerprint));
    let generation = RwSignal::new(0u32);

    // VSCode-like sidebar state (local to UI, not persisted).
    let sidebar_mode = RwSignal::new(SidebarMode::Hidden);
    // Local (not in Workspace) resizable width for the sidebar pane (Explorer or Games).
    // 0 means hidden (column collapses). Start hidden per spec.
    let explorer_width = RwSignal::new(0i32);
    let last_explorer_width = RwSignal::new(260i32);
    let is_dragging = RwSignal::new(false);
    let drag_start_x = RwSignal::new(0i32);
    let drag_start_w = RwSignal::new(0i32);

    // Lift modal state so both ActivityBar (Clear/Load) and Games panel can trigger the shared dialogs.
    let show_load = RwSignal::new(false);
    let warning = RwSignal::new(None::<String>);
    let warning_title = RwSignal::new(locale::localize(keys::WARNING_ERROR));

    // Global listeners for horizontal drag of sidebar width.
    Effect::new({
        let width_sig = explorer_width;
        let last_w = last_explorer_width;
        let drag_sig = is_dragging;
        let sx = drag_start_x;
        let sw = drag_start_w;
        move |_| {
            let window = web_sys::window().expect("window");
            let win_for_move = window.clone();
            let mmove = Closure::<dyn FnMut(WasmMouseEvent)>::new(move |ev: WasmMouseEvent| {
                if !drag_sig.get_untracked() {
                    return;
                }
                let dx = ev.client_x() - sx.get_untracked();
                let mut w = sw.get_untracked() + dx;
                if w < 0 {
                    w = 0;
                }
                let max_w = win_for_move
                    .inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .map(|ww| {
                        ((ww - 48.0 - f64::from(conf::ui::PROGRESS_RAY_WIDTH_PX)) / 2.0).floor()
                            as i32
                    })
                    .unwrap_or(400);
                if w > max_w {
                    w = max_w;
                }
                width_sig.set(w);
                if w > 0 {
                    last_w.set(w);
                }
            });
            let mup = Closure::<dyn FnMut(WasmMouseEvent)>::new(move |_ev: WasmMouseEvent| {
                drag_sig.set(false);
            });
            let _ = window
                .add_event_listener_with_callback("mousemove", mmove.as_ref().unchecked_ref());
            let _ =
                window.add_event_listener_with_callback("mouseup", mup.as_ref().unchecked_ref());
            mmove.forget();
            mup.forget();
        }
    });

    // autosave effect (unchanged)
    Effect::new(move |_| {
        workspace.vfs.with(|_| {});
        workspace.selected.with(|_| {});
        workspace.expanded.with(|_| {});
        let token = generation.get_untracked().wrapping_add(1);
        generation.set(token);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(conf::ui::AUTOSAVE_DEBOUNCE_MS).await;
            if generation.get_untracked() != token {
                return;
            }
            let _ = save_session(&workspace.snapshot());
            let (fp, encoded) = workspace
                .vfs
                .with_untracked(|vfs| (binaries_fingerprint(vfs), encode_binaries(vfs)));
            if fp != last_fp.get_untracked() {
                last_fp.set(fp);
                spawn_local(async move {
                    let _ = save_encoded(encoded).await;
                });
            }
        });
    });

    let ide_style = move || {
        format!(
            "--explorer-width: {}px; --progress-ray-width: {}px;",
            explorer_width.get(),
            conf::ui::PROGRESS_RAY_WIDTH_PX
        )
    };

    // Sidebar slot content (always emits a grid child in the explorer column position).
    // Hidden: empty .explorer div (width=0 from CSS var collapses it visually).
    // Explorer: the existing <Explorer/> (provides its own .explorer root + tree).
    // Games: dedicated panel (will provide .explorer root + Local/Global sections).
    let sidebar_slot = {
        let ws = workspace;
        let mode = sidebar_mode;
        let show_l = show_load;
        let warn = warning;
        let warn_t = warning_title;
        move || match mode.get() {
            SidebarMode::Hidden => view! { <div class="explorer"></div> }.into_any(),
            SidebarMode::Explorer => view! { <Explorer workspace=ws /> }.into_any(),
            SidebarMode::Games => view! {
                <GamesPanel
                    workspace=ws
                    show_load=show_l
                    warning=warn
                    warning_title=warn_t
                />
            }.into_any(),
        }
    };

    view! {
        <div class="ide" style=ide_style>
            <ActivityBar
                workspace=workspace
                sidebar_mode=sidebar_mode
                explorer_width=explorer_width
                last_explorer_width=last_explorer_width
                show_load=show_load
                warning=warning
                warning_title=warning_title
            />
            <ProgressRay workspace=workspace />
            {sidebar_slot}
            <div
                class="resizer"
                on:mousedown=move |ev: leptos::ev::MouseEvent| {
                    ev.prevent_default();
                    is_dragging.set(true);
                    drag_start_x.set(ev.client_x());
                    drag_start_w.set(explorer_width.get());
                }
            ></div>
            <Editor workspace=workspace />
        </div>
    }
}
