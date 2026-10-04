//! Bundled game-help Markdown copied into `help/` in the VFS.
//!
//! Same install rules as [`crate::help`]: copy when missing or when the stored
//! body is the app HTML shell. Do not overwrite user edits. Only the current
//! locale file is written; other locales already in the VFS stay.

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::Vfs;

const BUNDLED_EN: &str = include_str!("../game-help-en.md");
const BUNDLED_RU: &str = include_str!("../game-help-ru.md");

fn bundled(locale: &str) -> Option<&'static str> {
    match locale {
        "en" => Some(BUNDLED_EN),
        "ru" => Some(BUNDLED_RU),
        _ => None,
    }
}

/// Write the bundled game help for `locale` at [`conf::game_help::path`].
pub fn install_game_help(vfs: &mut Vfs, locale: &str) -> Result<(), String> {
    match bundled(locale) {
        Some(body) => vfs.put_file(&conf::game_help::path(locale), body.to_string()),
        None => Ok(()),
    }
}

/// Whether the VFS still needs a copy of the bundled game-help for `locale`.
pub fn needs_game_help(vfs: &Vfs, locale: &str) -> bool {
    if bundled(locale).is_none() {
        return false;
    }
    match vfs.read_file(&conf::game_help::path(locale)) {
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
    fn installs_only_current_locale() {
        let mut vfs = Vfs::default();
        assert!(needs_game_help(&vfs, "en"));
        assert!(!needs_game_help(&vfs, "de"));
        install_game_help(&mut vfs, "en").unwrap();
        assert!(vfs
            .read_file("help/game-help-en.md")
            .is_some_and(|body| body.starts_with('#')));
        assert!(!vfs.is_file("help/game-help-ru.md"));
        assert!(!vfs.is_file("game-help.md"));
        assert!(!needs_game_help(&vfs, "en"));
        assert!(needs_game_help(&vfs, "ru"));
    }

    #[test]
    fn other_locale_is_kept() {
        let mut vfs = Vfs::default();
        install_game_help(&mut vfs, "en").unwrap();
        install_game_help(&mut vfs, "ru").unwrap();
        assert!(vfs.is_file("help/game-help-en.md"));
        assert!(vfs.is_file("help/game-help-ru.md"));
        assert_ne!(
            vfs.read_file("help/game-help-en.md"),
            vfs.read_file("help/game-help-ru.md")
        );
        assert!(vfs
            .read_file("help/game-help-ru.md")
            .is_some_and(|body| body.contains("карточную игру")));
        assert!(!needs_game_help(&vfs, "ru"));
    }

    #[test]
    fn html_shell_is_reinstalled() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            &conf::game_help::path("en"),
            "<!DOCTYPE html>\n<html></html>".into(),
        )
        .unwrap();
        assert!(needs_game_help(&vfs, "en"));
        install_game_help(&mut vfs, "en").unwrap();
        assert!(vfs
            .read_file(&conf::game_help::path("en"))
            .is_some_and(|body| body.starts_with('#')));
    }

    #[test]
    fn user_edits_are_kept() {
        let mut vfs = Vfs::default();
        vfs.put_file(&conf::game_help::path("en"), "# custom\n".into())
            .unwrap();
        assert!(!needs_game_help(&vfs, "en"));
    }

    #[test]
    fn unknown_locale_does_not_invent_help() {
        let mut vfs = Vfs::default();
        install_game_help(&mut vfs, "de").unwrap();
        assert!(!vfs.is_file("help/game-help-de.md"));
    }
}
