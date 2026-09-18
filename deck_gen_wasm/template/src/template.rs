//! Install the `new-game` sample into the workspace VFS.
//!
//! Blobs come from GitHub only ([`crate::github`]). Deck `name` fields are
//! retargeted when the unique folder is not literally `new-game`. This module
//! owns install policy, not HTTP details.

use crate::github;
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::{unique_name, Vfs};
use progress_viewer::{progress_block, progress_loop, progress_wrapper, Progress};

/// Result of copying the template into the workspace.
pub struct InstalledGame {
    /// Folder name under `games/` (may be `new-game-N`).
    pub folder: String,
    /// [`conf::template::GITHUB_SOURCE_LABEL`].
    pub source: &'static str,
}

/// Fetch the template and write it under a unique `games/` folder.
pub async fn install_new_game(vfs: &mut Vfs, progress: Progress) -> Result<InstalledGame, String> {
    install_game(conf::template::GAME, vfs, progress).await
}

/// Install any game folder from under `games/{source_game}/` (e.g. "new-game", "monopoly-2.0").
/// The installed folder name may get -N suffix for uniqueness.
/// `games/conf.json5` is copied only if missing (shared).
pub async fn install_game(source_game: &str, vfs: &mut Vfs, progress: Progress) -> Result<InstalledGame, String> {
    progress_wrapper!(progress, {
        let (source, files) =
            progress_block!(progress, 0.0, 40.0, { load_game_files(source_game).await? });
        let folder = unique_game_folder(|name| vfs.exists(&format!("games/{name}")), source_game);
        let src_prefix = format!("games/{}/", source_game);
        progress_loop!(progress, 40.0, 100.0, files, |(path, content)| {
            if path == conf::template::GAMES_CONF {
                if !vfs.exists(conf::template::GAMES_CONF) {
                    vfs.put_file(conf::template::GAMES_CONF, content)?;
                }
            } else if let Some(rest) = path.strip_prefix(&src_prefix) {
                if !rest.is_empty() {
                    let dest = format!("games/{folder}/{rest}");
                    let content = retarget_game_id(&content, source_game, &folder);
                    vfs.put_file(&dest, content)?;
                }
            }
        });
        Ok(InstalledGame { folder, source })
    })
}

/// Unique folder under `games/`, starting at the provided base (e.g. "new-game" or "monopoly-2.0").
pub fn unique_game_folder(taken: impl Fn(&str) -> bool, base: &str) -> String {
    unique_name(base, taken)
}

fn retarget_game_id(content: &str, from: &str, to: &str) -> String {
    if from == to {
        return content.to_string();
    }
    content.replace(&format!("\"{from}."), &format!("\"{to}."))
}

async fn load_game_files(source_game: &str) -> Result<(&'static str, Vec<(String, String)>), String> {
    let paths = github::list_game_blob_paths(source_game).await?;
    let files = finish_game_files(source_game, github::fetch_listed_blobs(&paths).await?)?;
    Ok((conf::template::GITHUB_SOURCE_LABEL, files))
}

fn has_game_components(source_game: &str, files: &[(String, String)]) -> bool {
    let conf_ok = files
        .iter()
        .any(|(path, _)| path == conf::template::GAMES_CONF);
    let prefix = format!("games/{}/", source_game);
    let game = files.iter().any(|(path, _)| path.starts_with(&prefix));
    conf_ok && game
}

fn finish_game_files(source_game: &str, files: Vec<(String, String)>) -> Result<Vec<(String, String)>, String> {
    if !has_game_components(source_game, &files) {
        return Err(format!("missing games/{} or games/conf.json5", source_game));
    }
    Ok(files)
}

// Keep old list for compat (new-game specific filter used by legacy path if any).
#[allow(dead_code)]
async fn load_template_files() -> Result<(&'static str, Vec<(String, String)>), String> {
    let paths = github::list_template_blob_paths().await?;
    let files = finish_files(github::fetch_listed_blobs(&paths).await?)?;
    Ok((conf::template::GITHUB_SOURCE_LABEL, files))
}

#[allow(dead_code)]
fn has_both_components(files: &[(String, String)]) -> bool {
    let conf_ok = files
        .iter()
        .any(|(path, _)| path == conf::template::GAMES_CONF);
    let game = files
        .iter()
        .any(|(path, _)| path.starts_with(conf::template::PREFIX));
    conf_ok && game
}

#[allow(dead_code)]
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
        assert_eq!(unique_game_folder(|_| false, "new-game"), "new-game");
    }

    #[test]
    fn suffixes_when_taken() {
        let taken = |name: &str| name == "new-game" || name == "new-game-1";
        assert_eq!(unique_game_folder(taken, "new-game"), "new-game-2");
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
