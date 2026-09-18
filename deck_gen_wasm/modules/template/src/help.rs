//! Bundled help Markdown copied into the workspace root.
//!
//! [`crate::conf::help::PATH`] is `user-help.md` at the VFS root. The text is
//! the crate file compiled in with [`include_str`]; there is no GitHub GET.
//! The UI copies it when the path is missing or when the stored body is the
//! app HTML shell. Opening the preview tab stays in [`crate::workspace`].

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::Vfs;

const BUNDLED_HELP: &str = include_str!("../user-help.md");

/// Write the bundled help at [`conf::help::PATH`] (workspace root).
pub fn install_user_help(vfs: &mut Vfs) -> Result<(), String> {
    vfs.put_file(conf::help::PATH, BUNDLED_HELP.to_string())
}

/// Whether the VFS still needs a copy of the bundled help file.
pub fn needs_install(vfs: &Vfs) -> bool {
    match vfs.read_file(conf::help::PATH) {
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
    fn installs_at_workspace_root() {
        let mut vfs = Vfs::default();
        install_user_help(&mut vfs).unwrap();
        assert!(vfs
            .read_file("user-help.md")
            .is_some_and(|body| body.starts_with('#')));
        assert_eq!(
            vfs.read_file(conf::help::PATH),
            vfs.read_file("user-help.md")
        );
        assert!(!vfs.is_dir("deck_gen_wasm"));
        assert!(!needs_install(&vfs));
    }

    #[test]
    fn install_when_missing() {
        let vfs = Vfs::default();
        assert!(needs_install(&vfs));
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
        vfs.put_file(conf::help::PATH, "<!DOCTYPE html>\n<html></html>".into())
            .unwrap();
        assert!(needs_install(&vfs));
    }
}
