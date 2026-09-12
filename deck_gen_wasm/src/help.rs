//! Bundled help Markdown copied into the workspace root.
//!
//! [`crate::conf::help::PATH`] is `user-help.md` at the VFS root. The text is
//! the crate file compiled in with [`include_str`]; there is no GitHub GET.
//! The UI copies it when the path is missing or when the stored body is the
//! app HTML shell. Opening the preview tab stays in [`crate::workspace`].

use crate::conf;
use crate::fs::Vfs;

const BUNDLED_HELP: &str = include_str!("../user-help.md");

/// Markdown compiled into this WASM from `user-help.md`.
pub fn bundled_help() -> &'static str {
    BUNDLED_HELP
}

/// Write `content` at [`conf::help::PATH`] (workspace root).
pub fn install_user_help(vfs: &mut Vfs, content: String) -> Result<(), String> {
    vfs.put_file(conf::help::PATH, content)
}

/// Whether the VFS still needs a copy of the bundled help file.
pub fn needs_install(vfs: &Vfs) -> bool {
    match vfs.read_file(conf::help::PATH) {
        Some(body) => looks_like_html_document(body),
        None => true,
    }
}

/// Whether help preview should open (no editor tabs yet).
pub fn should_open_preview(tab_count: usize) -> bool {
    tab_count == 0
}

/// True when `body` is an HTML document, not Markdown help.
pub fn looks_like_html_document(body: &str) -> bool {
    let t = body.trim_start().to_ascii_lowercase();
    t.starts_with("<!doctype html") || t.starts_with("<html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_at_workspace_root() {
        let mut vfs = Vfs::default();
        install_user_help(&mut vfs, "# Hello".into()).unwrap();
        assert_eq!(vfs.read_file("user-help.md"), Some("# Hello"));
        assert_eq!(vfs.read_file(conf::help::PATH), Some("# Hello"));
        assert!(!vfs.is_dir("deck_gen_wasm"));
        assert!(!needs_install(&vfs));
    }

    #[test]
    fn install_when_missing_preview_when_no_tabs() {
        let vfs = Vfs::default();
        assert!(needs_install(&vfs));
        assert!(should_open_preview(0));
        assert!(!should_open_preview(1));
    }

    #[test]
    fn html_shell_is_not_help() {
        assert!(looks_like_html_document(
            "<!DOCTYPE html>\n<html lang=\"en\">"
        ));
        assert!(looks_like_html_document("  <html>"));
        assert!(!looks_like_html_document("# Deck generator (browser)\n"));
        assert!(!looks_like_html_document(BUNDLED_HELP));
    }

    #[test]
    fn reinstall_if_stored_help_is_html() {
        let mut vfs = Vfs::default();
        install_user_help(&mut vfs, "<!DOCTYPE html>\n<html></html>".into()).unwrap();
        assert!(needs_install(&vfs));
    }
}
