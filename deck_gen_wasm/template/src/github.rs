//! GitHub git-tree listing and raw file URLs for the sample game.
//!
//! The “new game” action needs every blob under `games/new-game/` plus
//! `games/conf.json5`. This module talks to the GitHub HTTP API and
//! `raw.githubusercontent.com`; it does not install files into the VFS — that
//! stays in [`crate::template`].

use serde::Deserialize;

use deck_gen_wasm_browser::fetch_text;
use deck_gen_wasm_conf as conf;

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
    let results = deck_gen_wasm_browser::map_join(
        paths.iter().cloned(),
        conf::io::FETCH_PARALLEL,
        |path| async move {
            fetch_text(&raw_url(&path))
                .await
                .ok()
                .map(|content| (path, content))
        },
    )
    .await;
    Ok(results.into_iter().flatten().collect())
}
