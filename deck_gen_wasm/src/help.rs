//! First-visit help Markdown: GitHub master, then the Trunk local copy.
//!
//! On an empty browser session the UI asks this module for
//! [`crate::conf::help::PATH`]. GitHub raw is tried first (same origin policy
//! as the `new-game` template). If that GET fails, the file Trunk copies next
//! to `index.html` is used. Installing into the VFS is a plain `put_file`;
//! opening the preview tab stays in [`crate::workspace`].

use crate::conf;
use crate::fs::Vfs;
use crate::github;
use crate::http::fetch_text;

/// Fetch help text from GitHub, or the local Trunk copy if GitHub fails.
pub async fn fetch_user_help() -> Result<String, String> {
    match fetch_text(&github::raw_url(conf::help::PATH)).await {
        Ok(body) => Ok(body),
        Err(github_err) => match fetch_text(conf::help::LOCAL_URL).await {
            Ok(body) => Ok(body),
            Err(local_err) => Err(format!(
                "Could not load {} from GitHub ({github_err}) or local copy ({local_err})",
                conf::help::PATH
            )),
        },
    }
}

/// Write help Markdown at [`conf::help::PATH`], creating parent folders.
pub fn install_user_help(vfs: &mut Vfs, content: String) -> Result<(), String> {
    vfs.put_file(conf::help::PATH, content)
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
    }
}
