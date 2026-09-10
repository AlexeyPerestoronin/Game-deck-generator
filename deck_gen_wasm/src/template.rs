//! Load the `new-game` sample from GitHub master, with a local Trunk copy as fallback.

use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::fs::Vfs;

const GITHUB_REPO: &str = "AlexeyPerestoronin/Game-deck-generator";
const GITHUB_BRANCH: &str = "master";
const TEMPLATE_GAME: &str = "new-game";
const TEMPLATE_PREFIX: &str = "games/new-game/";
const CONF_PATH: &str = "conf.json5";

const FALLBACK_PATHS: &[&str] = &[
    CONF_PATH,
    "games/new-game/vars/game.json",
    "games/new-game/vars/card.json",
    "games/new-game/decks/deck-1st/data.json5",
    "games/new-game/decks/deck-2nd/data.json5",
    "games/new-game/views/simple-front.html",
    "games/new-game/views/simple-front.scss",
    "games/new-game/views/simple-back.html",
    "games/new-game/views/simple-back.scss",
    "games/new-game/views/preview.html",
];

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
    let mut fetched_conf = None;
    let mut copied = 0usize;

    for (path, content) in files {
        if path == CONF_PATH {
            fetched_conf = Some(content);
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
        copied += 1;
    }

    if copied == 0 {
        return Err("The new-game template had no files to copy".into());
    }

    merge_conf(vfs, fetched_conf.as_deref(), &folder)?;
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
    match load_from_github().await {
        Ok(files) if has_template_files(&files) => Ok(("GitHub master", files)),
        github_result => match load_from_local().await {
            Ok(files) if has_template_files(&files) => Ok(("local template", files)),
            local_result => Err(format!(
                "Could not load new-game from GitHub ({}) or local copy ({})",
                describe(github_result),
                describe(local_result)
            )),
        },
    }
}

fn has_template_files(files: &[(String, String)]) -> bool {
    files
        .iter()
        .any(|(path, _)| path.starts_with(TEMPLATE_PREFIX))
}

fn describe(result: Result<Vec<(String, String)>, String>) -> String {
    match result {
        Ok(files) => format!("{} files, no new-game tree", files.len()),
        Err(err) => err,
    }
}

async fn load_from_github() -> Result<Vec<(String, String)>, String> {
    let tree_url = format!(
        "https://api.github.com/repos/{GITHUB_REPO}/git/trees/{GITHUB_BRANCH}?recursive=1"
    );
    let body = fetch_text(&tree_url).await?;
    let parsed: GithubTree = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    let mut paths: Vec<String> = parsed
        .tree
        .into_iter()
        .filter(|item| item.kind == "blob")
        .map(|item| item.path)
        .filter(|path| path == CONF_PATH || path.starts_with(TEMPLATE_PREFIX))
        .collect();
    if paths.is_empty() {
        paths = FALLBACK_PATHS.iter().map(|path| (*path).to_string()).collect();
    }
    let mut files = Vec::new();
    for path in paths {
        let url = format!(
            "https://raw.githubusercontent.com/{GITHUB_REPO}/{GITHUB_BRANCH}/{path}"
        );
        match fetch_text(&url).await {
            Ok(content) => files.push((path, content)),
            Err(err) if path == CONF_PATH => {
                return Err(err);
            }
            Err(_) => {}
        }
    }
    Ok(files)
}

async fn load_from_local() -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    for path in FALLBACK_PATHS {
        let url = format!("/template/{path}");
        let content = fetch_text(&url).await?;
        files.push(((*path).to_string(), content));
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

fn merge_conf(vfs: &mut Vfs, fetched: Option<&str>, game_id: &str) -> Result<(), String> {
    let existing = vfs.read_file(CONF_PATH).map(str::to_string);
    let base = existing
        .as_deref()
        .or(fetched)
        .ok_or_else(|| "Template did not include conf.json5".to_string())?;
    let mut value: Value = json5::from_str(base).map_err(|err| format!("conf.json5: {err}"))?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| "conf.json5 must be an object".to_string())?;
    let replace_games = existing.is_none();
    if replace_games {
        obj.insert("default_game".into(), json!(game_id));
    }

    let games = obj.entry("games").or_insert_with(|| json!({}));
    let games_obj = games
        .as_object_mut()
        .ok_or_else(|| "conf.json5 games must be an object".to_string())?;

    let mut spec = games_obj
        .get(TEMPLATE_GAME)
        .cloned()
        .unwrap_or_else(default_game_spec);
    if let Some(map) = spec.as_object_mut() {
        map.insert("dir".into(), json!(game_id));
    }
    if replace_games {
        games_obj.clear();
    }
    games_obj.insert(game_id.to_string(), spec);

    vfs.put_file(
        CONF_PATH,
        serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?,
    )
}

fn default_game_spec() -> Value {
    json!({
        "dir": TEMPLATE_GAME,
        "decks": "decks",
        "views": "views",
        "vars": "vars",
        "rules": "rules",
        "autogenerated": "_autogenerated",
        "duplex": "_autogenerated/a4-duplex"
    })
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
}
