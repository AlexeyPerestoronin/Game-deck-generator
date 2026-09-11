//! Install the `new-game` sample into the workspace VFS.
//!
//! GitHub is tried first ([`crate::github`]); if that set is incomplete, the
//! Trunk `/template/…` copy is used. Deck `name` fields are retargeted when the
//! unique folder is not literally `new-game`. This module owns install and
//! fallback policy, not HTTP details.

use crate::conf;
use crate::fs::{unique_name, Vfs};
use crate::github;
use crate::http::fetch_text;

/// Result of copying the template into the workspace.
pub struct InstalledGame {
    /// Folder name under `games/` (may be `new-game-N`).
    pub folder: String,
    /// [`conf::template::GITHUB_SOURCE_LABEL`] or [`conf::template::LOCAL_SOURCE_LABEL`].
    pub source: &'static str,
}

/// Fetch the template and write it under a unique `games/` folder.
pub async fn install_new_game(vfs: &mut Vfs) -> Result<InstalledGame, String> {
    let (source, files) = load_template_files().await?;
    let folder = unique_game_folder(|name| vfs.exists(&format!("games/{name}")));

    for (path, content) in files {
        if path == conf::template::GAMES_CONF {
            if !vfs.exists(conf::template::GAMES_CONF) {
                vfs.put_file(conf::template::GAMES_CONF, content)?;
            }
            continue;
        }
        let Some(rest) = path.strip_prefix(conf::template::PREFIX) else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        let dest = format!("games/{folder}/{rest}");
        let content = retarget_game_id(&content, conf::template::GAME, &folder);
        vfs.put_file(&dest, content)?;
    }

    Ok(InstalledGame { folder, source })
}

/// Unique folder under `games/`, starting at `new-game`.
pub fn unique_game_folder(taken: impl Fn(&str) -> bool) -> String {
    unique_name(conf::template::GAME, taken)
}

fn retarget_game_id(content: &str, from: &str, to: &str) -> String {
    if from == to {
        return content.to_string();
    }
    content.replace(&format!("\"{from}."), &format!("\"{to}."))
}

async fn load_template_files() -> Result<(&'static str, Vec<(String, String)>), String> {
    let listed = github::list_template_blob_paths().await;
    match load_from_github(&listed).await {
        Ok(files) => Ok((conf::template::GITHUB_SOURCE_LABEL, files)),
        github_result => match load_from_local(&listed).await {
            Ok(files) => Ok((conf::template::LOCAL_SOURCE_LABEL, files)),
            local_result => Err(format!(
                "Could not load new-game from GitHub ({}) or local copy ({})",
                describe(github_result),
                describe(local_result)
            )),
        },
    }
}

fn local_template_url(path: &str) -> String {
    // Relative to the page URL so GitHub Pages project sites (`/repo/…`) work.
    format!("template/{path}")
}

fn has_both_components(files: &[(String, String)]) -> bool {
    let conf_ok = files
        .iter()
        .any(|(path, _)| path == conf::template::GAMES_CONF);
    let game = files
        .iter()
        .any(|(path, _)| path.starts_with(conf::template::PREFIX));
    conf_ok && game
}

fn describe(result: Result<Vec<(String, String)>, String>) -> String {
    match result {
        Ok(files) => format!(
            "{} files, missing games/new-game or games/conf.json5",
            files.len()
        ),
        Err(err) => err,
    }
}

async fn load_from_github(
    listed: &Result<Vec<String>, String>,
) -> Result<Vec<(String, String)>, String> {
    let paths = listed.as_ref().map_err(|err| err.clone())?;
    finish_files(github::fetch_listed_blobs(paths).await?)
}

async fn load_from_local(
    listed: &Result<Vec<String>, String>,
) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    if let Ok(content) = fetch_text(&local_template_url(conf::template::GAMES_CONF)).await {
        files.push((conf::template::GAMES_CONF.to_string(), content));
    }
    if let Ok(paths) = listed {
        for path in paths {
            if !path.starts_with(conf::template::PREFIX) {
                continue;
            }
            if let Ok(content) = fetch_text(&local_template_url(path)).await {
                files.push((path.clone(), content));
            }
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
            conf::template::GAMES_CONF.to_string(),
            String::new()
        )]));
        assert!(!has_both_components(&[(
            "games/new-game/help.md".into(),
            String::new()
        )]));
        assert!(has_both_components(&[
            (conf::template::GAMES_CONF.to_string(), String::new()),
            ("games/new-game/help.md".into(), String::new()),
        ]));
    }
}
