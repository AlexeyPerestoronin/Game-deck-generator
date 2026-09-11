//! Load the `new-game` sample from GitHub master, with a local Trunk copy as fallback.

use gloo_net::http::Request;
use serde::Deserialize;

use crate::fs::Vfs;

const GITHUB_REPO: &str = "AlexeyPerestoronin/Game-deck-generator";
const GITHUB_BRANCH: &str = "master";
const TEMPLATE_GAME: &str = "new-game";
const TEMPLATE_PREFIX: &str = "games/new-game/";
const GAMES_CONF: &str = "games/conf.json5";

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

pub struct InstalledGame {
    pub folder: String,
    pub source: &'static str,
}

pub async fn install_new_game(vfs: &mut Vfs) -> Result<InstalledGame, String> {
    let (source, files) = load_template_files().await?;
    let folder = unique_game_folder(|name| vfs.exists(&format!("games/{name}")));

    for (path, content) in files {
        if path == GAMES_CONF {
            if !vfs.exists(GAMES_CONF) {
                vfs.put_file(GAMES_CONF, content)?;
            }
            continue;
        }
        let Some(rest) = path.strip_prefix(TEMPLATE_PREFIX) else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        let dest = format!("games/{folder}/{rest}");
        let content = retarget_game_id(&content, TEMPLATE_GAME, &folder);
        vfs.put_file(&dest, content)?;
    }

    Ok(InstalledGame { folder, source })
}

pub fn unique_game_folder(taken: impl Fn(&str) -> bool) -> String {
    if !taken(TEMPLATE_GAME) {
        return TEMPLATE_GAME.to_string();
    }
    let mut n = 1u32;
    loop {
        let name = format!("{TEMPLATE_GAME}-{n}");
        if !taken(&name) {
            return name;
        }
        n += 1;
    }
}

fn retarget_game_id(content: &str, from: &str, to: &str) -> String {
    if from == to {
        return content.to_string();
    }
    content.replace(&format!("\"{from}."), &format!("\"{to}."))
}

async fn load_template_files() -> Result<(&'static str, Vec<(String, String)>), String> {
    let listed = list_github_template_paths().await;
    match load_from_github(&listed).await {
        Ok(files) => Ok(("GitHub master", files)),
        github_result => match load_from_local(&listed).await {
            Ok(files) => Ok(("local template", files)),
            local_result => Err(format!(
                "Could not load new-game from GitHub ({}) or local copy ({})",
                describe(github_result),
                describe(local_result)
            )),
        },
    }
}

fn github_raw_url(path: &str) -> String {
    format!("https://raw.githubusercontent.com/{GITHUB_REPO}/{GITHUB_BRANCH}/{path}")
}

fn local_template_url(path: &str) -> String {
    format!("/template/{path}")
}

fn has_both_components(files: &[(String, String)]) -> bool {
    let conf = files.iter().any(|(path, _)| path == GAMES_CONF);
    let game = files.iter().any(|(path, _)| path.starts_with(TEMPLATE_PREFIX));
    conf && game
}

fn describe(result: Result<Vec<(String, String)>, String>) -> String {
    match result {
        Ok(files) => format!("{} files, missing games/new-game or games/conf.json5", files.len()),
        Err(err) => err,
    }
}

async fn list_github_template_paths() -> Result<Vec<String>, String> {
    let tree_url = format!(
        "https://api.github.com/repos/{GITHUB_REPO}/git/trees/{GITHUB_BRANCH}?recursive=1"
    );
    let body = fetch_text(&tree_url).await?;
    let parsed: GithubTree = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    Ok(parsed
        .tree
        .into_iter()
        .filter(|item| item.kind == "blob")
        .map(|item| item.path)
        .filter(|path| path == GAMES_CONF || path.starts_with(TEMPLATE_PREFIX))
        .collect())
}

async fn load_from_github(
    listed: &Result<Vec<String>, String>,
) -> Result<Vec<(String, String)>, String> {
    let paths = listed.as_ref().map_err(|err| err.clone())?;
    fetch_paths(paths, github_raw_url).await
}

async fn load_from_local(
    listed: &Result<Vec<String>, String>,
) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    if let Ok(content) = fetch_text(&local_template_url(GAMES_CONF)).await {
        files.push((GAMES_CONF.to_string(), content));
    }
    if let Ok(paths) = listed {
        for path in paths {
            if !path.starts_with(TEMPLATE_PREFIX) {
                continue;
            }
            if let Ok(content) = fetch_text(&local_template_url(path)).await {
                files.push((path.clone(), content));
            }
        }
    }
    finish_files(files)
}

async fn fetch_paths(
    paths: &[String],
    url: impl Fn(&str) -> String,
) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    for path in paths {
        if let Ok(content) = fetch_text(&url(path)).await {
            files.push((path.clone(), content));
        }
    }
    finish_files(files)
}

fn finish_files(files: Vec<(String, String)>) -> Result<Vec<(String, String)>, String> {
    if !has_both_components(&files) {
        return Err("missing games/new-game or games/conf.json5".into());
    }
    Ok(files)
}

async fn fetch_text(url: &str) -> Result<String, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|err| format!("{url}: {err}"))?;
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    response
        .text()
        .await
        .map_err(|err| format!("{url}: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_folder_is_new_game() {
        assert_eq!(unique_game_folder(|_| false), "new-game");
    }

    #[test]
    fn suffixes_when_taken() {
        let taken = |name: &str| name == "new-game" || name == "new-game-1";
        assert_eq!(unique_game_folder(taken), "new-game-2");
    }

    #[test]
    fn retargets_deck_name() {
        let src = r#""name": "new-game.deck-1st""#;
        assert_eq!(
            retarget_game_id(src, "new-game", "new-game-1"),
            r#""name": "new-game-1.deck-1st""#
        );
    }

    #[test]
    fn both_components_are_conf_and_folder() {
        assert!(!has_both_components(&[]));
        assert!(!has_both_components(&[(
            GAMES_CONF.to_string(),
            String::new()
        )]));
        assert!(!has_both_components(&[(
            "games/new-game/help.md".into(),
            String::new()
        )]));
        assert!(has_both_components(&[
            (GAMES_CONF.to_string(), String::new()),
            ("games/new-game/help.md".into(), String::new()),
        ]));
    }
}
