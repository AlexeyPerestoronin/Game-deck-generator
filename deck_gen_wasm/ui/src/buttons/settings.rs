//! Settings activity button (bottom of bar).
//! Opens a floating menu (reuses .context-menu styles + backdrop). Items delegate to feedback/theme/locale logic.
//! Menu positioned with getBoundingClientRect; no permanent .active on button.

use leptos::html;
use leptos::prelude::*;
use web_sys;

use crate::icons::SettingsIcon;
use crate::tooltips::DelayedTooltip;
use deck_gen_wasm_feedback::send_feedback_via_email;
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;

/// Simple position for the popup menu.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct MenuPos {
    left: f64,
    top: f64,
}

#[component]
pub fn SettingsButton() -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let menu_pos = RwSignal::new(MenuPos::default());
    let host: NodeRef<html::Div> = NodeRef::new();

    let tip: &'static str =
        Box::leak(locale::localize(keys::TOOLTIP_SETTINGS).into_boxed_str());

    let open_menu = move |_| {
        if let Some(el) = host.get() {
            let rect = el.get_bounding_client_rect();
            // Place to the right of button, vertically centered (like VSCode gear menu).
            let left = rect.right() + 4.0;
            let top = rect.top() + rect.height() / 2.0;
            menu_pos.set(MenuPos { left, top });
        }
        menu_open.set(!menu_open.get_untracked());
    };

    let close_menu = move || {
        menu_open.set(false);
    };

    let do_feedback = move |_| {
        let _ = send_feedback_via_email();
        close_menu();
    };

    let do_theme = move |_| {
        let _ = crate::theme::cycle_and_apply();
        close_menu();
    };

    let do_locale = move |_| {
        let _ = locale::change_locale();
        close_menu();
        // reload to apply (same as old LocaleButton)
        if let Some(win) = web_sys::window() {
            let _ = win.location().reload();
        }
    };

    // Close on outside click (simple: listen on document? For direct impl use backdrop like context).
    view! {
        <div class="tooltip-host" node_ref=host>
            <DelayedTooltip text=tip>
                <button
                    class="activity-btn"
                    aria-label=move || locale::localize(keys::ARIA_SETTINGS)
                    on:click=open_menu
                >
                    <SettingsIcon />
                </button>
            </DelayedTooltip>

            <Show when=move || menu_open.get()>
                <div
                    class="context-backdrop"
                    on:click=move |_| close_menu()
                >
                    <div
                        class="context-menu settings-menu"
                        role="menu"
                        style=move || {
                            let p = menu_pos.get();
                            format!("left:{}px;top:{}px; transform: translateY(-50%);", p.left, p.top)
                        }
                        on:click=move |ev| ev.stop_propagation()
                    >
                        <button class="context-item" role="menuitem" on:click=do_feedback>
                            {move || locale::localize(keys::MENU_SETTINGS_FEEDBACK)}
                        </button>
                        <button class="context-item" role="menuitem" on:click=do_theme>
                            {move || locale::localize(keys::MENU_SETTINGS_THEME)}
                        </button>
                        <button class="context-item" role="menuitem" on:click=do_locale>
                            {move || locale::localize(keys::MENU_SETTINGS_LOCALE)}
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
