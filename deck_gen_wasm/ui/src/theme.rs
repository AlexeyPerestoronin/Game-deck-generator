//! Color theme (dark/light/system) with localStorage and CSS dataset.
//! Start as System. Cycle Dark→Light→System. System follows OS via matchMedia.

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::js_sys::{Function, Reflect};
use web_sys::window;

use deck_gen_wasm_conf as conf;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ColorTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl ColorTheme {
    pub fn next(self) -> Self {
        match self {
            ColorTheme::Dark => ColorTheme::Light,
            ColorTheme::Light => ColorTheme::System,
            ColorTheme::System => ColorTheme::Dark,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ColorTheme::Dark => "dark",
            ColorTheme::Light => "light",
            ColorTheme::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => ColorTheme::Dark,
            "light" => ColorTheme::Light,
            _ => ColorTheme::System,
        }
    }
}

pub fn load() -> ColorTheme {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            if let Ok(Some(v)) = storage.get_item(conf::session::THEME_KEY) {
                return ColorTheme::from_str(&v);
            }
        }
    }
    ColorTheme::System
}

fn load_stored() -> ColorTheme {
    load()
}

fn save_stored(t: ColorTheme) {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(conf::session::THEME_KEY, t.as_str());
        }
    }
}

fn effective_str(t: ColorTheme) -> &'static str {
    match t {
        ColorTheme::Dark => "dark",
        ColorTheme::Light => "light",
        ColorTheme::System => {
            let dark = if let Some(_win) = window() {
                // Use Function to invoke matchMedia so we do not require extra web_sys features.
                let call = Function::new_with_args(
                    "q",
                    "try { return window.matchMedia(q).matches; } catch(e){ return false; }",
                );
                call.call1(&JsValue::UNDEFINED, &"(prefers-color-scheme: dark)".into())
                    .ok()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true)
            } else {
                true
            };
            if dark { "dark" } else { "light" }
        }
    }
}

fn set_dataset(eff: &str) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-theme", eff);
            }
        }
    }
}

static mut MEDIA_LISTENER: Option<Closure<dyn FnMut()>> = None;

/// Apply stored theme (or System) to <html data-theme="dark|light">.
/// For System also attaches a one-time change listener that re-applies on OS change
/// while the mode remains System. Call once on mount.
pub fn apply() {
    let theme = load_stored();
    let eff = effective_str(theme);
    set_dataset(eff);

    // Setup OS listener once if not already (for System mode updates without reload).
    // SAFETY: single init at app start in browser main thread.
    if unsafe { MEDIA_LISTENER.is_none() } {
        if let Some(_win) = window() {
            // Obtain mql via JS Function (avoids gated web_sys::match_media)
            let get_mql = Function::new_with_args(
                "q",
                "try { return window.matchMedia(q); } catch(e){ return null; }",
            );
            if let Ok(mql) = get_mql.call1(&JsValue::UNDEFINED, &"(prefers-color-scheme: dark)".into()) {
                if !mql.is_null() {
                    let cb = Closure::<dyn FnMut()>::new(move || {
                        // Re-read; if still System, update to current effective.
                        let current = load_stored();
                        if current == ColorTheme::System {
                            let eff2 = effective_str(current);
                            set_dataset(eff2);
                        }
                    });
                    // addEventListener via Reflect to avoid needing MediaQueryList type
                    if let Ok(add_fn) = Reflect::get(&mql, &JsValue::from_str("addEventListener")) {
                        if let Ok(add_fn) = add_fn.dyn_into::<Function>() {
                            let _ = add_fn.call2(&mql, &"change".into(), cb.as_ref().unchecked_ref());
                        }
                    }
                    // keep the closure from being dropped
                    unsafe { MEDIA_LISTENER = Some(cb); }
                }
            }
        }
    }
}

/// Cycle to next theme, persist, apply effective dataset, return the new chosen theme.
pub fn cycle_and_apply() -> ColorTheme {
    let next = load_stored().next();
    save_stored(next);
    let eff = effective_str(next);
    set_dataset(eff);
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_cycles_three_states() {
        let d = ColorTheme::Dark;
        let l = d.next();
        let s = l.next();
        let d2 = s.next();
        assert_eq!(l, ColorTheme::Light);
        assert_eq!(s, ColorTheme::System);
        assert_eq!(d2, ColorTheme::Dark);
        // full cycle
        assert_eq!(d.next().next().next(), d);
    }
}
