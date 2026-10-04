//! Bundled user-help Markdown copied into `help/` in the VFS.
//!
//! [`deck_gen_wasm_conf::help::path`] is `help/user-help-<locale>.md`. The text is
//! compiled in with [`include_str`]; there is no GitHub GET.
//! Copy when the path is missing or the stored body is the app HTML shell.

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::Vfs;

const BUNDLED_EN: &str = include_str!("../user-help-en.md");
const BUNDLED_RU: &str = include_str!("../user-help-ru.md");

fn bundled(locale: &str) -> Option<&'static str> {
    match locale {
        "en" => Some(BUNDLED_EN),
        "ru" => Some(BUNDLED_RU),
        _ => None,
    }
}

/// Write the bundled user help for `locale` at [`conf::help::path`].
pub fn install_user_help(vfs: &mut Vfs, locale: &str) -> Result<(), String> {
    match bundled(locale) {
        Some(body) => vfs.put_file(&conf::help::path(locale), body.to_string()),
        None => Ok(()),
    }
}

/// Whether the VFS still needs a copy of the bundled user-help for `locale`.
pub fn needs_install(vfs: &Vfs, locale: &str) -> bool {
    if bundled(locale).is_none() {
        return false;
    }
    match vfs.read_file(&conf::help::path(locale)) {
        Some(body) => looks_like_html_document(body),
        None => true,
    }
}

/// True when the start of `body` is an HTML document, not Markdown help.
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
        assert!(needs_install(&vfs, "en"));
        assert!(!needs_install(&vfs, "de"));
        install_user_help(&mut vfs, "en").unwrap();
        assert!(vfs
            .read_file("help/user-help-en.md")
            .is_some_and(|body| body.starts_with('#')));
        assert_eq!(
            vfs.read_file(&conf::help::path("en")),
            vfs.read_file("help/user-help-en.md")
        );
        assert!(!vfs.is_file("help/user-help-ru.md"));
        assert!(!vfs.is_file("user-help.md"));
        assert!(!needs_install(&vfs, "en"));
        assert!(needs_install(&vfs, "ru"));
    }

    #[test]
    fn other_locale_is_kept() {
        let mut vfs = Vfs::default();
        install_user_help(&mut vfs, "en").unwrap();
        install_user_help(&mut vfs, "ru").unwrap();
        assert!(vfs.is_file("help/user-help-en.md"));
        assert!(vfs.is_file("help/user-help-ru.md"));
        assert_ne!(
            vfs.read_file("help/user-help-en.md"),
            vfs.read_file("help/user-help-ru.md")
        );
        assert!(vfs
            .read_file("help/user-help-ru.md")
            .is_some_and(|body| body.contains("рабочая область")));
        assert!(!needs_install(&vfs, "en"));
        assert!(!needs_install(&vfs, "ru"));
    }

    #[test]
    fn unknown_locale_does_not_invent_help() {
        let mut vfs = Vfs::default();
        install_user_help(&mut vfs, "de").unwrap();
        assert!(!vfs.is_file("help/user-help-de.md"));
        assert!(!vfs.is_dir("help"));
    }

    #[test]
    fn html_shell_is_not_help() {
        assert!(looks_like_html_document(
            "<!DOCTYPE html>\n<html lang=\"en\">"
        ));
        assert!(looks_like_html_document("  <html>"));
        assert!(!looks_like_html_document("# Deck generator (browser)\n"));
        assert!(!looks_like_html_document(BUNDLED_EN));
        assert!(!looks_like_html_document(BUNDLED_RU));
    }

    #[test]
    fn reinstall_if_stored_help_is_html() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            &conf::help::path("en"),
            "<!DOCTYPE html>\n<html></html>".into(),
        )
        .unwrap();
        assert!(needs_install(&vfs, "en"));
        install_user_help(&mut vfs, "en").unwrap();
        assert!(vfs
            .read_file(&conf::help::path("en"))
            .is_some_and(|body| body.starts_with('#')));
    }

    #[test]
    fn user_edits_are_kept() {
        let mut vfs = Vfs::default();
        vfs.put_file(&conf::help::path("en"), "# custom\n".into())
            .unwrap();
        assert!(!needs_install(&vfs, "en"));
    }
}
