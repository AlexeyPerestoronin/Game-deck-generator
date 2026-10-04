//! Copy bundled model json5 + locale ai-help into the VFS when missing or HTML-shell.

use deck_gen_wasm_conf as wconf;
use deck_gen_wasm_fs::Vfs;

const GEMINI_FLASH: &str = include_str!("../bundled/gemini-2.0-flash.json5");
const GEMINI_LITE: &str = include_str!("../bundled/gemini-2.0-flash-lite.json5");
const DEEPSEEK: &str = include_str!("../bundled/deepseek-chat.json5");
const GROK: &str = include_str!("../bundled/grok-3-mini.json5");
const AI_HELP_EN: &str = include_str!("../bundled/ai-help-en.md");
const AI_HELP_RU: &str = include_str!("../bundled/ai-help-ru.md");

fn bundled_models() -> [(&'static str, &'static str); 4] {
    [
        ("ai-models/gemini-2.0-flash.json5", GEMINI_FLASH),
        ("ai-models/gemini-2.0-flash-lite.json5", GEMINI_LITE),
        ("ai-models/deepseek-chat.json5", DEEPSEEK),
        ("ai-models/grok-3-mini.json5", GROK),
    ]
}

fn bundled_ai_help(locale: &str) -> Option<&'static str> {
    match locale {
        "en" => Some(AI_HELP_EN),
        "ru" => Some(AI_HELP_RU),
        _ => None,
    }
}

/// True when any default model json5 is missing/HTML-shell, or the current
/// locale's ai-help needs a copy. Other locales are ignored.
pub fn needs_ai_install(vfs: &Vfs, locale: &str) -> bool {
    bundled_models()
        .iter()
        .any(|(path, _)| file_needs_install(vfs, path))
        || bundled_ai_help(locale).is_some_and(|_| file_needs_install(vfs, &wconf::ai::help(locale)))
}

/// Write missing / HTML-shell defaults. Does not overwrite a user-edited json5
/// (e.g. a key) or a user-edited help file. Does not remove other locales.
pub fn install_ai_defaults(vfs: &mut Vfs, locale: &str) -> Result<(), String> {
    vfs.mkdir(wconf::ai::DIR)?;
    vfs.mkdir(wconf::ai::LOG_DIR)?;
    for (path, body) in bundled_models() {
        if file_needs_install(vfs, path) {
            vfs.put_file(path, body.to_string())?;
        }
    }
    if let Some(body) = bundled_ai_help(locale) {
        let path = wconf::ai::help(locale);
        if file_needs_install(vfs, &path) {
            vfs.put_file(&path, body.to_string())?;
        }
    }
    Ok(())
}

fn file_needs_install(vfs: &Vfs, path: &str) -> bool {
    match vfs.read_file(path) {
        Some(body) => looks_like_html_document(body),
        None => true,
    }
}

fn looks_like_html_document(body: &str) -> bool {
    let t = body.trim_start();
    let n = t.len().min(32);
    let prefix = t.get(..n).unwrap_or(t).to_ascii_lowercase();
    prefix.starts_with("<!doctype html") || prefix.starts_with("<html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_files_are_copied_for_current_locale_only() {
        let mut vfs = Vfs::default();
        assert!(needs_ai_install(&vfs, "en"));
        install_ai_defaults(&mut vfs, "en").unwrap();
        assert!(vfs.is_file("ai-models/gemini-2.0-flash.json5"));
        assert!(vfs.is_file("ai-models/gemini-2.0-flash-lite.json5"));
        assert!(vfs.is_file("ai-models/deepseek-chat.json5"));
        assert!(vfs.is_file("ai-models/grok-3-mini.json5"));
        assert!(vfs.is_file(&wconf::ai::help("en")));
        assert!(!vfs.is_file(&wconf::ai::help("ru")));
        assert!(!vfs.is_file("ai-models/ai-help.md"));
        assert!(vfs.is_dir(wconf::ai::LOG_DIR));
        assert!(!needs_ai_install(&vfs, "en"));
        assert!(needs_ai_install(&vfs, "ru"));
        let key_file = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(key_file.contains("api_key: \"\"") || key_file.contains("api_key: ''"));
    }

    #[test]
    fn other_locale_help_is_kept() {
        let mut vfs = Vfs::default();
        install_ai_defaults(&mut vfs, "en").unwrap();
        install_ai_defaults(&mut vfs, "ru").unwrap();
        assert!(vfs.is_file(&wconf::ai::help("en")));
        assert!(vfs.is_file(&wconf::ai::help("ru")));
        assert_ne!(
            vfs.read_file(&wconf::ai::help("en")),
            vfs.read_file(&wconf::ai::help("ru"))
        );
        assert!(vfs
            .read_file(&wconf::ai::help("ru"))
            .is_some_and(|body| body.contains("Конфиги моделей")));
        assert!(!needs_ai_install(&vfs, "ru"));
    }

    #[test]
    fn user_key_is_not_overwritten() {
        let mut vfs = Vfs::default();
        install_ai_defaults(&mut vfs, "en").unwrap();
        vfs.put_file(
            "ai-models/gemini-2.0-flash.json5",
            "{ id: \"gemini-2.0-flash\", api_key: \"SECRET\" }".into(),
        )
        .unwrap();
        assert!(!needs_ai_install(&vfs, "en"));
        install_ai_defaults(&mut vfs, "en").unwrap();
        let body = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(body.contains("SECRET"));
    }

    #[test]
    fn custom_ai_help_is_not_overwritten() {
        let mut vfs = Vfs::default();
        let path = wconf::ai::help("en");
        vfs.put_file(&path, "# my help\n".into()).unwrap();
        install_ai_defaults(&mut vfs, "en").unwrap();
        assert_eq!(vfs.read_file(&path), Some("# my help\n"));
    }

    #[test]
    fn html_shell_is_reinstalled() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            "ai-models/gemini-2.0-flash.json5",
            "<!DOCTYPE html>\n<html></html>".into(),
        )
        .unwrap();
        assert!(file_needs_install(&vfs, "ai-models/gemini-2.0-flash.json5"));
        install_ai_defaults(&mut vfs, "en").unwrap();
        let body = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(body.contains("gemini-2.0-flash"));
        assert!(!looks_like_html_document(body));
    }

    #[test]
    fn html_shell_ai_help_is_reinstalled() {
        let mut vfs = Vfs::default();
        let path = wconf::ai::help("en");
        vfs.put_file(&path, "<!DOCTYPE html>\n<html></html>".into())
            .unwrap();
        assert!(needs_ai_install(&vfs, "en"));
        install_ai_defaults(&mut vfs, "en").unwrap();
        assert!(vfs.read_file(&path).is_some_and(|body| body.starts_with('#')));
    }

    #[test]
    fn unknown_locale_does_not_invent_help() {
        let mut vfs = Vfs::default();
        install_ai_defaults(&mut vfs, "de").unwrap();
        assert!(vfs.is_file("ai-models/gemini-2.0-flash.json5"));
        assert!(!vfs.is_file("help/ai-help-de.md"));
        assert!(!vfs.is_file(&wconf::ai::help("en")));
    }
}
