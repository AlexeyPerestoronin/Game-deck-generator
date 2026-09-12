//! Help Markdown: GitHub master, then the file compiled into this crate.
//!
//! The UI asks this module for [`crate::conf::help::PATH`] when that file is
//! missing from the VFS or when the stored body is the app’s HTML shell (Trunk
//! SPA fallback). GitHub raw is tried first. If that GET fails or returns HTML,
//! [`include_str`] of `user-help.md` is used. Installing into the VFS is a
//! plain `put_file`; opening the preview tab stays in [`crate::workspace`].

use crate::conf;
use crate::fs::Vfs;
use crate::github;
use crate::http::fetch_text;

const BUNDLED_HELP: &str = include_str!("../user-help.md");

/// Fetch help text from GitHub, or the Markdown compiled into this WASM.
pub async fn fetch_user_help() -> Result<String, String> {
    match fetch_text(&github::raw_url(conf::help::PATH)).await {
        Ok(body) if !looks_like_html_document(&body) => Ok(body),
        github_result => {
            if looks_like_html_document(BUNDLED_HELP) {
                return Err(format!(
                    "Could not load {} from GitHub ({}) and bundled copy is not Markdown",
                    conf::help::PATH,
                    describe_github(github_result)
                ));
            }
            Ok(BUNDLED_HELP.to_string())
        }
    }
}

fn describe_github(result: Result<String, String>) -> String {
    match result {
        Ok(_) => "got HTML instead of Markdown".into(),
        Err(err) => err,
    }
}

/// Write help Markdown at [`conf::help::PATH`], creating parent folders.
pub fn install_user_help(vfs: &mut Vfs, content: String) -> Result<(), String> {
    vfs.put_file(conf::help::PATH, content)
}

/// Whether the VFS still needs a download of [`conf::help::PATH`].
pub fn needs_download(vfs: &Vfs) -> bool {
    match vfs.read_file(conf::help::PATH) {
        Some(body) => looks_like_html_document(body),
        None => true,
    }
}

/// Whether help preview should open (no editor tabs yet).
pub fn should_open_preview(tab_count: usize) -> bool {
    tab_count == 0
}

/// Trunk’s SPA fallback returns `index.html` with HTTP 200 for unknown paths.
pub fn looks_like_html_document(body: &str) -> bool {
    let t = body.trim_start().to_ascii_lowercase();
    t.starts_with("<!doctype html") || t.starts_with("<html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_at_configured_path() {
        let mut vfs = Vfs::default();
        install_user_help(&mut vfs, "# Hello".into()).unwrap();
        assert_eq!(vfs.read_file(conf::help::PATH), Some("# Hello"));
        assert!(vfs.is_dir("deck_gen_wasm"));
        assert!(vfs.is_file("deck_gen_wasm/user-help.md"));
        assert!(!needs_download(&vfs));
    }

    #[test]
    fn download_when_missing_preview_when_no_tabs() {
        let vfs = Vfs::default();
        assert!(needs_download(&vfs));
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
    fn redownload_if_stored_help_is_html() {
        let mut vfs = Vfs::default();
        install_user_help(&mut vfs, "<!DOCTYPE html>\n<html></html>".into()).unwrap();
        assert!(needs_download(&vfs));
    }
}
