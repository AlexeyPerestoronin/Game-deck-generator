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
pub(crate) struct GithubTree {
    tree: Vec<GithubTreeItem>,
}

#[derive(Deserialize)]
pub(crate) struct GithubTreeItem {
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
#[allow(dead_code)]
pub async fn list_template_blob_paths() -> Result<Vec<String>, String> {
    list_game_blob_paths(conf::template::GAME).await
}

/// Recursive git-tree blob paths for a specific game folder + the shared conf.
pub async fn list_game_blob_paths(game_folder: &str) -> Result<Vec<String>, String> {
    let tree_url = format!(
        "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
        conf::github::REPO,
        conf::github::BRANCH
    );
    let body = fetch_text(&tree_url).await?;
    let parsed: GithubTree = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    let prefix = format!("games/{}/", game_folder);
    Ok(parsed
        .tree
        .into_iter()
        .filter(|item| item.kind == "blob")
        .map(|item| item.path)
        .filter(|path| path == conf::template::GAMES_CONF || path.starts_with(&prefix))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree(paths: &[(&str, &str)]) -> GithubTree {
        GithubTree {
            tree: paths
                .iter()
                .map(|(p, k)| GithubTreeItem {
                    path: p.to_string(),
                    kind: k.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn filters_only_top_level_game_trees() {
        let t = make_tree(&[
            ("games/new-game", "tree"),
            ("games/monopoly-2.0", "tree"),
            ("games/new-game/conf.json5", "blob"),
            ("games/monopoly-2.0/decks/foo", "tree"),
            ("other/stuff", "tree"),
            ("games/conf.json5", "blob"),
        ]);
        let folders = filter_game_folders_from_tree(&t);
        assert_eq!(folders, vec!["monopoly-2.0", "new-game"]);
    }

    #[test]
    fn ignores_blobs_and_nested() {
        let t = make_tree(&[
            ("games/foo/bar", "tree"),
            ("games/foo", "blob"),
        ]);
        let folders = filter_game_folders_from_tree(&t);
        assert!(folders.is_empty());
    }
}

/// List top-level game folder names under `games/` (type=="tree", exactly `games/<name>`).
/// Used by Global section in Games sidebar. Does not fetch file contents.
pub async fn list_game_folders() -> Result<Vec<String>, String> {
    let tree_url = format!(
        "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
        conf::github::REPO,
        conf::github::BRANCH
    );
    let body = fetch_text(&tree_url).await?;
    let parsed: GithubTree = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    let mut folders: Vec<String> = filter_game_folders_from_tree(&parsed);
    folders.sort();
    Ok(folders)
}

/// Pure filter: given parsed tree, return top-level game folder names under games/.
pub fn filter_game_folders_from_tree(tree: &GithubTree) -> Vec<String> {
    let mut folders: Vec<String> = tree
        .tree
        .iter()
        .filter(|item| item.kind == "tree")
        .filter_map(|item| {
            if let Some(rest) = item.path.strip_prefix("games/") {
                if !rest.is_empty() && !rest.contains('/') {
                    return Some(rest.to_string());
                }
            }
            None
        })
        .collect();
    folders.sort();
    folders
}
