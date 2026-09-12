//! GitHub git-tree listing and raw file URLs.
//!
//! The “new game” action needs every blob under `games/new-game/` plus
//! `games/conf.json5`. First visit also pulls [`crate::conf::help::PATH`] via
//! [`raw_url`]. This module talks to the GitHub HTTP API and
//! `raw.githubusercontent.com`; it does not install files into the VFS — that
//! stays in [`crate::template`] and [`crate::help`].

use serde::Deserialize;

use crate::conf;
use crate::http::fetch_text;

#[derive(Deserialize)]
struct GithubTree {
    tree: Vec<GithubTreeItem>,
}

#[derive(Deserialize)]
struct GithubTreeItem {
    path: String,
    #[serde(rename = "type")]
    kind: String,
}

/// `raw.githubusercontent.com` URL for `path` on the configured branch.
pub fn raw_url(path: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/{}/{}/{path}",
        conf::github::REPO,
        conf::github::BRANCH
    )
}

/// Recursive git-tree blob paths that belong to the sample game (or games conf).
pub async fn list_template_blob_paths() -> Result<Vec<String>, String> {
    let tree_url = format!(
        "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
        conf::github::REPO,
        conf::github::BRANCH
    );
    let body = fetch_text(&tree_url).await?;
    let parsed: GithubTree = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    Ok(parsed
        .tree
        .into_iter()
        .filter(|item| item.kind == "blob")
        .map(|item| item.path)
        .filter(|path| {
            path == conf::template::GAMES_CONF || path.starts_with(conf::template::PREFIX)
        })
        .collect())
}

/// Fetch each listed path from GitHub raw; skip individual HTTP failures.
pub async fn fetch_listed_blobs(paths: &[String]) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    for path in paths {
        if let Ok(content) = fetch_text(&raw_url(path)).await {
            files.push((path.clone(), content));
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_raw_url_uses_master_and_repo_path() {
        let url = raw_url(conf::help::PATH);
        assert!(
            url.contains(&format!(
                "raw.githubusercontent.com/{}/{}",
                conf::github::REPO,
                conf::github::BRANCH
            )),
            "{url}"
        );
        assert!(url.ends_with("/deck_gen_wasm/user-help.md"), "{url}");
    }
}
