//! Bundled game-help Markdown copied into the workspace root.
//!
//! Same install rules as [`crate::help`]: copy when missing or when the stored
//! body is the app HTML shell. Do not overwrite user edits.

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::Vfs;

const BUNDLED_GAME_HELP: &str = include_str!("../game-help.md");

/// Write the bundled game help at [`conf::game_help::PATH`].
pub fn install_game_help(vfs: &mut Vfs) -> Result<(), String> {
    vfs.put_file(conf::game_help::PATH, BUNDLED_GAME_HELP.to_string())
}

/// Whether the VFS still needs a copy of the bundled game-help file.
pub fn needs_game_help(vfs: &Vfs) -> bool {
    match vfs.read_file(conf::game_help::PATH) {
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
    fn installs_at_workspace_root() {
        let mut vfs = Vfs::default();
        assert!(needs_game_help(&vfs));
        install_game_help(&mut vfs).unwrap();
        assert!(vfs
            .read_file("game-help.md")
            .is_some_and(|body| body.starts_with('#')));
        assert!(!needs_game_help(&vfs));
    }

    #[test]
    fn html_shell_is_reinstalled() {
        let mut vfs = Vfs::default();
        vfs.put_file(conf::game_help::PATH, "<!DOCTYPE html>\n<html></html>".into())
            .unwrap();
        assert!(needs_game_help(&vfs));
        install_game_help(&mut vfs).unwrap();
        assert!(vfs
            .read_file(conf::game_help::PATH)
            .is_some_and(|body| body.starts_with('#')));
    }

    #[test]
    fn user_edits_are_kept() {
        let mut vfs = Vfs::default();
        vfs.put_file(conf::game_help::PATH, "# custom\n".into())
            .unwrap();
        assert!(!needs_game_help(&vfs));
    }
}
