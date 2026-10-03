//! Copy bundled model json5 + ai-help.md into the VFS when missing or HTML-shell.

use deck_gen_wasm_conf as wconf;
use deck_gen_wasm_fs::Vfs;

const GEMINI_FLASH: &str = include_str!("../bundled/gemini-2.0-flash.json5");
const GEMINI_LITE: &str = include_str!("../bundled/gemini-2.0-flash-lite.json5");
const DEEPSEEK: &str = include_str!("../bundled/deepseek-chat.json5");
const GROK: &str = include_str!("../bundled/grok-3-mini.json5");
const AI_HELP: &str = include_str!("../bundled/ai-help.md");

fn bundled() -> [(&'static str, &'static str); 5] {
    [
        ("ai-models/gemini-2.0-flash.json5", GEMINI_FLASH),
        ("ai-models/gemini-2.0-flash-lite.json5", GEMINI_LITE),
        ("ai-models/deepseek-chat.json5", DEEPSEEK),
        ("ai-models/grok-3-mini.json5", GROK),
        (wconf::ai::HELP, AI_HELP),
    ]
}

/// True when any default AI file is missing or looks like the app HTML shell.
pub fn needs_ai_install(vfs: &Vfs) -> bool {
    bundled()
        .iter()
        .any(|(path, _)| file_needs_install(vfs, path))
}

/// Write missing / HTML-shell defaults. Does not overwrite a user-edited json5 (e.g. a key).
pub fn install_ai_defaults(vfs: &mut Vfs) -> Result<(), String> {
    vfs.mkdir(wconf::ai::DIR)?;
    vfs.mkdir(wconf::ai::LOG_DIR)?;
    for (path, body) in bundled() {
        if file_needs_install(vfs, path) {
            vfs.put_file(path, body.to_string())?;
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
    fn missing_files_are_copied() {
        let mut vfs = Vfs::default();
        assert!(needs_ai_install(&vfs));
        install_ai_defaults(&mut vfs).unwrap();
        assert!(vfs.is_file("ai-models/gemini-2.0-flash.json5"));
        assert!(vfs.is_file("ai-models/gemini-2.0-flash-lite.json5"));
        assert!(vfs.is_file("ai-models/deepseek-chat.json5"));
        assert!(vfs.is_file("ai-models/grok-3-mini.json5"));
        assert!(vfs.is_file(wconf::ai::HELP));
        assert!(vfs.is_dir(wconf::ai::LOG_DIR));
        assert!(!needs_ai_install(&vfs));
        let key_file = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(key_file.contains("api_key: \"\"") || key_file.contains("api_key: ''"));
    }

    #[test]
    fn user_key_is_not_overwritten() {
        let mut vfs = Vfs::default();
        install_ai_defaults(&mut vfs).unwrap();
        vfs.put_file(
            "ai-models/gemini-2.0-flash.json5",
            "{ id: \"gemini-2.0-flash\", api_key: \"SECRET\" }".into(),
        )
        .unwrap();
        assert!(!needs_ai_install(&vfs));
        install_ai_defaults(&mut vfs).unwrap();
        let body = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(body.contains("SECRET"));
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
        install_ai_defaults(&mut vfs).unwrap();
        let body = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(body.contains("gemini-2.0-flash"));
        assert!(!looks_like_html_document(body));
    }
}
