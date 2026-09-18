//! PNG-based icons (32×32 source, displayed 16×16) for activity-bar buttons.
//! Three (or four for split) state images switched purely by CSS on the parent .activity-btn.
//! No inline SVG and no currentColor remain. State PNGs live in deck_gen_wasm/icons/buttons/.

use leptos::prelude::*;

#[component]
pub fn DownloadIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/download/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/download/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/download/click.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn LoadGameIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/load_game/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/load_game/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/load_game/click.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn NewGameIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/new_game/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/new_game/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/new_game/click.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn PreparePdfIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/prepare_pdf/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/prepare_pdf/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/prepare_pdf/click.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn PrepareHtmlIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/prepare_html/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/prepare_html/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/prepare_html/click.drawio.png" width="40" height="40" />
        </span>
    }
}

/// Split icon renders four frames. CSS uses .state-active when the button carries `.active`.
#[component]
pub fn SplitPreviewIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"    src="icons/buttons/split_preview/off.drawio.png"    width="40" height="40" />
            <img class="state-on"     src="icons/buttons/split_preview/on.drawio.png"     width="40" height="40" />
            <img class="state-click"  src="icons/buttons/split_preview/click.drawio.png"  width="40" height="40" />
            <img class="state-active" src="icons/buttons/split_preview/active.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn ClearIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/clear/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/clear/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/clear/click.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn FeedbackIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/feedback/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/feedback/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/feedback/click.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn ThemeIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off dark"   src="icons/buttons/theme/dark-off.drawio.png"   width="40" height="40" />
            <img class="state-on dark"    src="icons/buttons/theme/dark-on.drawio.png"    width="40" height="40" />
            <img class="state-click dark" src="icons/buttons/theme/dark-click.drawio.png" width="40" height="40" />
            <img class="state-off light"   src="icons/buttons/theme/light-off.drawio.png"   width="40" height="40" />
            <img class="state-on light"    src="icons/buttons/theme/light-on.drawio.png"    width="40" height="40" />
            <img class="state-click light" src="icons/buttons/theme/light-click.drawio.png" width="40" height="40" />
            <img class="state-off system"   src="icons/buttons/theme/system-off.drawio.png"   width="40" height="40" />
            <img class="state-on system"    src="icons/buttons/theme/system-on.drawio.png"    width="40" height="40" />
            <img class="state-click system" src="icons/buttons/theme/system-click.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn LocaleIcon() -> impl IntoView {
    // Reactive: src chosen from current locale so icon flips without remount.
    // CSS .state-* still drives off/on/click via parent button classes.
    view! {
        <span class="activity-icon" aria-hidden="true">
            {move || {
                let prefix = match deck_gen_wasm_locale::get_active_locale() {
                    deck_gen_wasm_locale::Locale::En => "en",
                    deck_gen_wasm_locale::Locale::Ru => "ru",
                };
                view! {
                    <img class="state-off"   src=format!("icons/buttons/locale/{}-off.drawio.png", prefix)   width="40" height="40" />
                    <img class="state-on"    src=format!("icons/buttons/locale/{}-on.drawio.png", prefix)    width="40" height="40" />
                    <img class="state-click" src=format!("icons/buttons/locale/{}-click.drawio.png", prefix) width="40" height="40" />
                }
            }}
        </span>
    }
}

#[component]
pub fn ExplorerIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"    src="icons/buttons/explorer/off.drawio.png"    width="40" height="40" />
            <img class="state-on"     src="icons/buttons/explorer/on.drawio.png"     width="40" height="40" />
            <img class="state-click"  src="icons/buttons/explorer/click.drawio.png"  width="40" height="40" />
            <img class="state-active" src="icons/buttons/explorer/active.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn GamesIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"    src="icons/buttons/games/off.drawio.png"    width="40" height="40" />
            <img class="state-on"     src="icons/buttons/games/on.drawio.png"     width="40" height="40" />
            <img class="state-click"  src="icons/buttons/games/click.drawio.png"  width="40" height="40" />
            <img class="state-active" src="icons/buttons/games/active.drawio.png" width="40" height="40" />
        </span>
    }
}

#[component]
pub fn SettingsIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/settings/off.drawio.png"   width="40" height="40" />
            <img class="state-on"    src="icons/buttons/settings/on.drawio.png"    width="40" height="40" />
            <img class="state-click" src="icons/buttons/settings/click.drawio.png" width="40" height="40" />
        </span>
    }
}
