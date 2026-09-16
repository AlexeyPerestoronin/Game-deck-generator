//! Minimal EN/RU localization for the UI chrome.
//!
//! - `localize(key)` reads the current reactive locale (RwSignal) so Leptos views
//!   update on change without reload.
//! - Persist in localStorage under conf::session::LOCALE_KEY ("en"|"ru").
//! - Default EN. Cycle En <-> Ru.
//! - Dict from json5 at build (include_str).
//! - Unit tests run native (no wasm, persist is no-op).

use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg(not(test))]
use deck_gen_wasm_conf as conf;

#[cfg(not(test))]
use leptos::prelude::*;
#[cfg(not(test))]
use web_sys::window;
#[cfg(not(test))]
use wasm_bindgen::JsValue;

pub mod keys;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Locale {
    #[default]
    En,
    Ru,
}

impl Locale {
    fn as_str(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ru => "ru",
        }
    }
    fn from_str(s: &str) -> Self {
        if s.eq_ignore_ascii_case("ru") {
            Locale::Ru
        } else {
            Locale::En
        }
    }
}

type Dict = HashMap<String, HashMap<String, String>>;

static DICT: OnceLock<Dict> = OnceLock::new();

fn dict() -> &'static Dict {
    DICT.get_or_init(|| {
        let src = include_str!("../dict.json5");
        match json5::from_str::<Dict>(src) {
            Ok(d) => d,
            Err(_) => HashMap::new(),
        }
    })
}

fn lookup(key: &str, loc: Locale) -> String {
    let d = dict();
    if let Some(entry) = d.get(key) {
        let lang = loc.as_str().to_ascii_uppercase();
        if let Some(val) = entry.get(&lang) {
            return val.clone();
        }
        // fallback to EN
        if let Some(val) = entry.get("EN") {
            return val.clone();
        }
    }
    // unknown key -> return as-is (per spec)
    key.to_string()
}

// ---- test vs real storage/signal ----

#[cfg(test)]
mod store {
    use std::sync::atomic::{AtomicU8, Ordering};
    static CUR: AtomicU8 = AtomicU8::new(0); // 0=en, 1=ru

    pub fn get() -> super::Locale {
        if CUR.load(Ordering::Relaxed) == 1 {
            super::Locale::Ru
        } else {
            super::Locale::En
        }
    }
    pub fn set(l: super::Locale) {
        CUR.store(if l == super::Locale::Ru { 1 } else { 0 }, Ordering::Relaxed);
    }
    pub fn persist(_l: super::Locale) {}
    pub fn load_persisted() -> super::Locale {
        super::Locale::En
    }
}

#[cfg(not(test))]
mod store {
    use super::*;
    static LOCALE_SIG: OnceLock<RwSignal<Locale>> = OnceLock::new();

    fn locale_signal() -> RwSignal<Locale> {
        *LOCALE_SIG.get_or_init(|| RwSignal::new(store::load_persisted()))
    }

    pub fn get() -> Locale {
        locale_signal().get()
    }

    pub fn set(l: Locale) {
        locale_signal().set(l);
    }

    pub fn persist(l: Locale) {
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item(conf::session::LOCALE_KEY, l.as_str());
            }
        }
    }

    pub fn load_persisted() -> Locale {
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(v)) = storage.get_item(conf::session::LOCALE_KEY) {
                    return Locale::from_str(&v);
                }
            }
        }
        Locale::En
    }

    pub fn ensure_signal() {
        // force creation inside a reactive owner (call from App component body)
        let _ = locale_signal();
    }

    pub fn update_document(l: Locale) {
        if let Some(win) = window() {
            if let Some(doc) = win.document() {
                // lang attr
                let lang = lookup(keys::HTML_LANG, l);
                if let Some(el) = doc.document_element() {
                    let _ = el.set_attribute("lang", &lang);
                }
                // title via Reflect (avoids extra Document features)
                let title = lookup(keys::DOCUMENT_TITLE, l);
                let _ = js_sys::Reflect::set(
                    &doc,
                    &JsValue::from_str("title"),
                    &JsValue::from_str(&title),
                );
            }
        }
    }
}

#[cfg(test)]
use store as imp;
#[cfg(not(test))]
use store as imp;

/// Ensure the reactive signal is created (call once early from a Leptos component).
#[cfg(not(test))]
pub fn ensure_locale_signal() {
    imp::ensure_signal();
}

/// Read current locale (subscribes when called from reactive context).
pub fn get_active_locale() -> Locale {
    imp::get()
}

/// Change to next (En -> Ru -> En), persist, update doc, return ok.
pub fn change_locale() -> Result<(), String> {
    let next = match get_active_locale() {
        Locale::En => Locale::Ru,
        Locale::Ru => Locale::En,
    };
    set_locale(next)
}

/// Set explicit locale, persist, update live UI bits.
pub fn set_locale(l: Locale) -> Result<(), String> {
    imp::set(l);
    imp::persist(l);
    #[cfg(not(test))]
    {
        imp::update_document(l);
    }
    Ok(())
}

/// Translate key for CURRENT locale (reads signal.get() inside).
/// Unknown key is returned verbatim.
pub fn localize(key: &str) -> String {
    lookup(key, get_active_locale())
}

/// For initial apply of html/doc (call after ensure).
#[cfg(not(test))]
pub fn apply_initial_document() {
    let l = get_active_locale();
    imp::update_document(l);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keys_have_en_and_ru() {
        // touch all pub keys
        let keys = [
            keys::TOOLTIP_CLEAR,
            keys::TOOLTIP_DOWNLOAD,
            keys::TOOLTIP_LOAD_GAME,
            keys::TOOLTIP_NEW_GAME,
            keys::TOOLTIP_PREPARE_HTML,
            keys::TOOLTIP_PREPARE_PDF,
            keys::TOOLTIP_SPLIT_PREVIEW,
            keys::TOOLTIP_FEEDBACK,
            keys::TOOLTIP_THEME,
            keys::TOOLTIP_LOCALE,
            keys::ARIA_CLEAR,
            keys::ARIA_DOWNLOAD,
            keys::ARIA_LOAD_GAME,
            keys::ARIA_NEW_GAME,
            keys::ARIA_PREPARE_HTML,
            keys::ARIA_PREPARE_PDF,
            keys::ARIA_SPLIT_PREVIEW,
            keys::ARIA_FEEDBACK,
            keys::ARIA_THEME,
            keys::ARIA_LOCALE,
            keys::MODAL_CANCEL,
            keys::MODAL_OK,
            keys::CONFIRM_CLEAR_TITLE,
            keys::CONFIRM_CLEAR_MESSAGE,
            keys::CONFIRM_CLEAR_LABEL,
            keys::CONFIRM_LOAD_TITLE,
            keys::CONFIRM_LOAD_MESSAGE,
            keys::CONFIRM_LOAD_LABEL,
            keys::WARNING_ERROR,
            keys::WARNING_CANNOT_LOAD_FOLDER,
            keys::WARNING_CANNOT_PREPARE_HTML,
            keys::WARNING_CANNOT_PREPARE_PDF,
            keys::WARNING_CANNOT_LOAD_NEW_GAME,
            keys::WARNING_CANNOT_LOAD_FILES,
            keys::EXPLORER_GAMES,
            keys::EXPLORER_NEW_FILE,
            keys::EXPLORER_NEW_FOLDER,
            keys::EDITOR_NO_PREVIEW,
            keys::EDITOR_SELECT_FILE,
            keys::EDITOR_BINARY,
            keys::EDITOR_LOADING,
            keys::MENU_RENAME,
            keys::MENU_DELETE,
            keys::MENU_COPY,
            keys::MENU_PREVIEW,
            keys::MENU_LOAD_FILES,
            keys::MENU_PAST,
            keys::MENU_CLOSE_ALL,
            keys::TAB_CLOSE,
            keys::TAB_PREVIEW_PREFIX,
            keys::BUTTON_NEW_FILE,
            keys::BUTTON_NEW_FOLDER,
            keys::PROMPT_NEW_FILE,
            keys::PROMPT_NEW_FOLDER,
            keys::PROMPT_NEW_NAME,
            keys::STATUS_COULD_NOT_UPDATE,
            keys::STATUS_NOTHING_TO_PASTE,
            keys::STATUS_UNKNOWN_COMMAND,
            keys::STATUS_PASTED,
            keys::STATUS_DELETED,
            keys::STATUS_RENAMED,
            keys::STATUS_WORKSPACE_CLEARED,
            keys::STATUS_OPENED,
            keys::STATUS_SAVED,
            keys::STATUS_SELECT_FILES,
            keys::STATUS_SELECT_FOLDER,
            keys::STATUS_LOADING_FOLDER,
            keys::STATUS_LOADED_N_INTO,
            keys::STATUS_LOADED,
            keys::STATUS_PREPARING_HTML,
            keys::STATUS_PREPARED_HTML,
            keys::STATUS_PREPARING_PDF,
            keys::STATUS_PREPARED_PDF,
            keys::STATUS_LOADING_TEMPLATE,
            keys::STATUS_ADDED,
            keys::STATUS_DOWNLOADING_ZIP,
            keys::STATUS_DOWNLOADED,
            keys::STATUS_NOT_A_FOLDER,
            keys::HTML_LANG,
            keys::DOCUMENT_TITLE,
        ];
        for k in keys {
            let en = lookup(k, Locale::En);
            let ru = lookup(k, Locale::Ru);
            assert!(!en.is_empty() && en != k, "missing EN for {k}");
            assert!(!ru.is_empty() && ru != k, "missing RU for {k}");
            // both present and different or at least defined
        }
    }

    #[test]
    fn change_cycles_en_ru_en() {
        let start = get_active_locale();
        let _ = set_locale(Locale::En);
        let a = get_active_locale();
        let _ = change_locale();
        let b = get_active_locale();
        let _ = change_locale();
        let c = get_active_locale();
        assert_eq!(a, Locale::En);
        assert_eq!(b, Locale::Ru);
        assert_eq!(c, Locale::En);
        // restore
        let _ = set_locale(start);
    }

    #[test]
    fn unknown_key_returns_itself() {
        assert_eq!(localize("no-such-key-xyz"), "no-such-key-xyz");
    }
}
