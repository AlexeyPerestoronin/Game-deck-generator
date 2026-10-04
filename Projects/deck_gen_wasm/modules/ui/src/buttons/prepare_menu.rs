//! Shared activity-bar Prepare HTML/PDF control: tooltip or checkbox menu.

use std::collections::HashSet;

use gloo_timers::future::TimeoutFuture;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::buttons::ActivityButton;
use crate::icons::{PrepareHtmlIcon, PreparePdfIcon};
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_workspace::{game_menu_name, local_games, Workspace};

/// Which prepare pipeline the control runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrepareKind {
    /// `prepare_html`.
    Html,
    /// `prepare_pdf`.
    Pdf,
}

/// Activity-bar button that scopes generation to selected local games.
#[component]
pub fn PrepareScopedButton(
    workspace: Workspace,
    warning: RwSignal<Option<String>>,
    warning_title: RwSignal<String>,
    kind: PrepareKind,
) -> impl IntoView {
    let (title_key, tooltip_key, aria_key) = match kind {
        PrepareKind::Html => (
            keys::WARNING_CANNOT_PREPARE_HTML,
            keys::TOOLTIP_PREPARE_HTML,
            keys::ARIA_PREPARE_HTML,
        ),
        PrepareKind::Pdf => (
            keys::WARNING_CANNOT_PREPARE_PDF,
            keys::TOOLTIP_PREPARE_PDF,
            keys::ARIA_PREPARE_PDF,
        ),
    };
    let tip: &'static str = Box::leak(locale::localize(tooltip_key).into_boxed_str());
    let aria = Signal::derive(move || locale::localize(aria_key));
    let disabled = Signal::derive(move || workspace.loading.get());
    let checked = RwSignal::new(HashSet::<String>::new());
    let menu_open = RwSignal::new(false);
    let hide_gen = RwSignal::new(0u32);
    let host: NodeRef<html::Div> = NodeRef::new();
    let pos = RwSignal::new((0.0f64, 0.0f64));

    let games = move || workspace.vfs.with(|vfs| local_games(vfs));
    let many = move || games().len() >= 2;

    let run = move || {
        warning_title.set(locale::localize(title_key));
        let list = games();
        if list.is_empty() {
            warning.set(Some(locale::localize(keys::WARNING_NO_GAMES)));
            return;
        }
        let roots: Vec<String> = if list.len() == 1 {
            vec![list[0].root.clone()]
        } else {
            let selected = checked.get();
            list.into_iter()
                .filter(|g| selected.contains(&g.root))
                .map(|g| g.root)
                .collect()
        };
        if roots.is_empty() {
            warning.set(Some(locale::localize(keys::WARNING_NO_GAME_SELECTED)));
            return;
        }
        match kind {
            PrepareKind::Html => workspace.prepare_html_for(&roots, warning),
            PrepareKind::Pdf => workspace.prepare_pdf_for(&roots, warning),
        }
    };

    let show_menu = move || {
        if let Some(el) = host.get() {
            let rect = el.get_bounding_client_rect();
            let left = rect.right() + 8.0;
            let top = rect.top();
            pos.set(clamp_menu(
                left,
                top,
                220.0,
                (games().len() as f64) * 28.0 + 8.0,
            ));
        }
        hide_gen.update(|n| *n = n.wrapping_add(1));
        menu_open.set(true);
    };

    let schedule_hide = move || {
        let token = hide_gen.get_untracked().wrapping_add(1);
        hide_gen.set(token);
        spawn_local(async move {
            TimeoutFuture::new(200).await;
            if hide_gen.get_untracked() == token {
                menu_open.set(false);
            }
        });
    };

    view! {
        {move || {
            if many() {
                view! {
                    <div
                        class="tooltip-host"
                        node_ref=host
                        on:mouseenter=move |_| show_menu()
                        on:mouseleave=move |_| schedule_hide()
                    >
                        <button
                            class="activity-btn"
                            aria-label=aria
                            disabled=disabled
                            on:click=move |_| run()
                        >
                            {match kind {
                                PrepareKind::Html => view! { <PrepareHtmlIcon /> }.into_any(),
                                PrepareKind::Pdf => view! { <PreparePdfIcon /> }.into_any(),
                            }}
                        </button>
                        <Show when=move || menu_open.get()>
                            <div
                                class="prepare-game-menu"
                                style=move || {
                                    let (left, top) = pos.get();
                                    format!("left:{}px;top:{}px", left, top)
                                }
                                on:mouseenter=move |_| {
                                    hide_gen.update(|n| *n = n.wrapping_add(1));
                                    menu_open.set(true);
                                }
                                on:mouseleave=move |_| schedule_hide()
                            >
                                <For
                                    each=move || games()
                                    key=|g| g.root.clone()
                                    children=move |g| {
                                        let root = g.root.clone();
                                        let root_check = root.clone();
                                        view! {
                                            <label class="prepare-game-item">
                                                <input
                                                    type="checkbox"
                                                    prop:checked=move || checked.get().contains(&root_check)
                                                    on:change=move |ev| {
                                                        let on = event_target_checked(&ev);
                                                        checked.update(|set| {
                                                            if on {
                                                                set.insert(root.clone());
                                                            } else {
                                                                set.remove(&root);
                                                            }
                                                        });
                                                    }
                                                />
                                                <span>{move || game_menu_name(&g)}</span>
                                            </label>
                                        }
                                    }
                                />
                            </div>
                        </Show>
                    </div>
                }
                .into_any()
            } else {
                view! {
                    <ActivityButton
                        tip=tip
                        aria_label=aria
                        disabled=disabled
                        on_click=run
                    >
                        <Show
                            when=move || kind == PrepareKind::Html
                            fallback=|| view! { <PreparePdfIcon /> }
                        >
                            <PrepareHtmlIcon />
                        </Show>
                    </ActivityButton>
                }
                .into_any()
            }
        }}
    }
}

fn clamp_menu(left: f64, top: f64, width: f64, height: f64) -> (f64, f64) {
    let (vw, vh) = web_sys::window()
        .and_then(|w| {
            Some((
                w.inner_width().ok()?.as_f64()?,
                w.inner_height().ok()?.as_f64()?,
            ))
        })
        .unwrap_or((800.0, 600.0));
    let left = left.min(vw - width - 8.0).max(8.0);
    let top = top.min(vh - height - 8.0).max(8.0);
    (left, top)
}
