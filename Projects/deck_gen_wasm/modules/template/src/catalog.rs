//! Deck-Games catalog: detect game roots, parse `info.json5`, fetch posters, install games.
//!
//! Pure helpers are unit-tested natively. HTTP lives behind the same GitHub
//! tree/raw helpers as `new-game`.

use std::collections::HashMap;

use crate::github;
use crate::template::{retarget_game_id, unique_game_folder};
use deck_gen_wasm_browser::{fetch_bytes, fetch_text, map_join};
use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::{file_name, Vfs};
use progress_viewer::{progress_block, progress_loop, progress_wrapper, Progress};

/// Bundled poster used when a game has no `preview` file.
pub const DEFAULT_PREVIEW_HTML: &str =
    include_str!("../../../template/default-preview.html");

/// Bundled 1×1 gray PNG used when a game has no icon file.
///
/// Bytes of `projects/deck_gen_wasm/template/default-icon.png` (tiny placeholder).
pub const DEFAULT_ICON_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x00, 0x03, 0x00, 0x01, 0x18, 0x57, 0x0D, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// Parsed `{game-root}/rules/preview/info.json5`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameInfo {
    /// File name of the poster HTML in the same folder.
    pub preview: Option<String>,
    /// File name of the icon in the same folder.
    pub icon: Option<String>,
    /// `info.name` map (`EN`, `RU`, …).
    pub names: HashMap<String, String>,
    /// Search tags; empty when missing or not an array.
    pub tags: Vec<String>,
}

impl GameInfo {
    /// Localized name: `lang` (e.g. `"EN"` / `"RU"`), then `"EN"`.
    pub fn name_for_locale(&self, lang: &str) -> Option<&str> {
        non_empty(self.names.get(lang)).or_else(|| non_empty(self.names.get("EN")))
    }
}

fn non_empty(value: Option<&String>) -> Option<&str> {
    value.map(String::as_str).filter(|s| !s.is_empty())
}

/// One published game in the Deck-Games catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogEntry {
    /// Repo-relative game root (e.g. `Games/Poker`).
    pub root: String,
    /// Parsed `info.json5`.
    pub info: GameInfo,
}

/// Directories that contain `game.json5`, sorted by path.
pub fn game_roots_from_paths<I, S>(paths: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let suffix = format!("/{}", conf::catalog::GAME_MARKER);
    let marker = conf::catalog::GAME_MARKER;
    let mut roots: Vec<String> = paths
        .into_iter()
        .filter_map(|path| {
            let path = path.as_ref();
            if path == marker {
                Some(String::new())
            } else {
                path.strip_suffix(&suffix).map(str::to_string)
            }
        })
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

/// Catalog game roots: marker blobs under `Games/`.
pub fn catalog_game_roots(paths: &[String]) -> Vec<String> {
    let games = conf::catalog::GAMES_DIR;
    let prefix = format!("{games}/");
    game_roots_from_paths(paths.iter())
        .into_iter()
        .filter(|root| root == games || root.starts_with(&prefix))
        .collect()
}

/// Parse `info.json5`. `None` means the document is not valid JSON5 / not an object.
pub fn parse_info_json5(text: &str) -> Option<GameInfo> {
    let value: serde_json::Value = json5::from_str(text).ok()?;
    let obj = value.as_object()?;
    let preview = string_field(obj.get("preview"));
    let icon = string_field(obj.get("icon"));
    let mut names = HashMap::new();
    let mut tags = Vec::new();
    if let Some(info) = obj.get("info").and_then(|v| v.as_object()) {
        if let Some(name) = info.get("name").and_then(|v| v.as_object()) {
            for (key, val) in name {
                if let Some(s) = val.as_str().filter(|s| !s.is_empty()) {
                    names.insert(key.clone(), s.to_string());
                }
            }
        }
        if let Some(arr) = info.get("tags").and_then(|v| v.as_array()) {
            tags = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
    }
    Some(GameInfo {
        preview,
        icon,
        names,
        tags,
    })
}

fn string_field(value: Option<&serde_json::Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Case-insensitive match on `name.EN`, `name.RU`, and `tags`. Empty query matches all.
pub fn info_matches_query(info: &GameInfo, query: &str) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return true;
    }
    let q = q.to_ascii_lowercase();
    let name_hit = ["EN", "RU"].iter().any(|key| {
        info.names
            .get(*key)
            .is_some_and(|name| name.to_ascii_lowercase().contains(&q))
    });
    let tag_hit = info
        .tags
        .iter()
        .any(|tag| tag.to_ascii_lowercase().contains(&q));
    name_hit || tag_hit
}

enum BlobBody {
    Text(String),
    Bytes(Vec<u8>),
}

const TEXT_EXTS: &[&str] = &["html", "htm", "json", "json5", "md", "css", "scss", "js"];

fn ext_of(path: &str) -> String {
    file_name(path)
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default()
}

fn is_text_blob(path: &str) -> bool {
    TEXT_EXTS.contains(&ext_of(path).as_str())
}

async fn fetch_blob(repo: &str, branch: &str, path: String) -> Result<(String, BlobBody), String> {
    let url = github::raw_url_for(repo, branch, &path);
    if is_text_blob(&path) {
        let text = fetch_text(&url).await?;
        Ok((path, BlobBody::Text(text)))
    } else {
        let bytes = fetch_bytes(&url).await?;
        Ok((path, BlobBody::Bytes(bytes)))
    }
}

fn preview_prefix(root: &str) -> String {
    if root.is_empty() {
        format!("{}/", conf::catalog::PREVIEW_DIR)
    } else {
        format!("{}/{}/", root, conf::catalog::PREVIEW_DIR)
    }
}

fn info_path(root: &str) -> String {
    if root.is_empty() {
        format!("{}/{}", conf::catalog::PREVIEW_DIR, conf::catalog::INFO_FILE)
    } else {
        format!(
            "{}/{}/{}",
            root,
            conf::catalog::PREVIEW_DIR,
            conf::catalog::INFO_FILE
        )
    }
}

/// Download catalog posters into a fresh VFS and return visible games.
pub async fn load_catalog() -> Result<(Vfs, Vec<CatalogEntry>), String> {
    let repo = conf::catalog::REPO;
    let branch = conf::catalog::BRANCH;
    let blobs = github::fetch_tree_blob_paths(repo, branch).await?;
    let roots = catalog_game_roots(&blobs);
    let mut by_root: Vec<(String, Vec<String>)> = roots
        .into_iter()
        .map(|root| {
            let prefix = preview_prefix(&root);
            let files: Vec<String> = blobs
                .iter()
                .filter(|path| path.starts_with(&prefix))
                .cloned()
                .collect();
            (root, files)
        })
        .collect();
    let results = map_join(
        by_root.drain(..),
        conf::io::FETCH_PARALLEL,
        |(root, files)| async move { fetch_one_catalog_game(repo, branch, root, files).await },
    )
    .await;
    let mut vfs = Vfs::default();
    let mut entries = Vec::new();
    for item in results.into_iter().flatten() {
        let (files, entry) = item;
        for (path, body) in files {
            match body {
                BlobBody::Text(text) => {
                    let _ = vfs.put_file(&path, text);
                }
                BlobBody::Bytes(bytes) => {
                    let _ = vfs.put_bytes(&path, bytes);
                }
            }
        }
        entries.push(entry);
    }
    entries.sort_by(|a, b| a.root.cmp(&b.root));
    Ok((vfs, entries))
}

async fn fetch_one_catalog_game(
    repo: &str,
    branch: &str,
    root: String,
    files: Vec<String>,
) -> Option<(Vec<(String, BlobBody)>, CatalogEntry)> {
    if files.is_empty() {
        return None;
    }
    let info = info_path(&root);
    if !files.iter().any(|path| path == &info) {
        return None;
    }
    let fetched = map_join(files, conf::io::FETCH_PARALLEL, |path| async move {
        fetch_blob(repo, branch, path).await
    })
    .await;
    let mut out = Vec::new();
    for item in fetched {
        match item {
            Ok(pair) => out.push(pair),
            Err(_) => return None,
        }
    }
    let info_body = out.iter().find(|(path, _)| path == &info)?;
    let text = match &info_body.1 {
        BlobBody::Text(text) => text.as_str(),
        BlobBody::Bytes(_) => return None,
    };
    let parsed = parse_info_json5(text)?;
    Some((
        out,
        CatalogEntry {
            root,
            info: parsed,
        },
    ))
}

/// Download every blob under `game_root` into persist-VFS as `games/{unique}/…`.
pub async fn install_catalog_game(
    vfs: &mut Vfs,
    game_root: &str,
    progress: Progress,
) -> Result<crate::template::InstalledGame, String> {
    progress_wrapper!(progress, {
        let repo = conf::catalog::REPO;
        let branch = conf::catalog::BRANCH;
        let blobs = progress_block!(progress, 0.0, 20.0, {
            github::fetch_tree_blob_paths(repo, branch).await?
        });
        let prefix = format!("{game_root}/");
        let paths: Vec<String> = blobs
            .into_iter()
            .filter(|path| path.starts_with(&prefix))
            .collect();
        if paths.is_empty() {
            return Err(format!("no files under {game_root}"));
        }
        let fetched = progress_block!(progress, 20.0, 70.0, {
            let results = map_join(paths, conf::io::FETCH_PARALLEL, |path| async move {
                fetch_blob(repo, branch, path).await.ok()
            })
            .await;
            results.into_iter().flatten().collect::<Vec<_>>()
        });
        if fetched.is_empty() {
            return Err(format!("failed to download {game_root}"));
        }
        let base = file_name(game_root);
        let folder = unique_game_folder(|name| vfs.exists(&format!("games/{name}")), base);
        progress_loop!(progress, 70.0, 100.0, fetched, |(path, body)| {
            if let Some(rest) = path.strip_prefix(&prefix) {
                if !rest.is_empty() {
                    let dest = format!("games/{folder}/{rest}");
                    match body {
                        BlobBody::Text(text) => {
                            let text = retarget_game_id(&text, base, &folder);
                            vfs.put_file(&dest, text)?;
                        }
                        BlobBody::Bytes(bytes) => {
                            vfs.put_bytes(&dest, bytes)?;
                        }
                    }
                }
            }
        });
        Ok(crate::template::InstalledGame {
            folder,
            source: conf::catalog::REPO,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_roots_from_marker_paths() {
        let paths = [
            "Games/a/game.json5",
            "Games/a/rules/preview/info.json5",
            "Games/nested/b/game.json5",
            "Games/skip.txt",
            "other/game.json5",
            "Games/game.json5",
        ];
        let roots = game_roots_from_paths(paths);
        assert_eq!(
            roots,
            vec!["Games", "Games/a", "Games/nested/b", "other"]
        );
        assert_eq!(
            catalog_game_roots(&paths.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
            vec!["Games", "Games/a", "Games/nested/b"]
        );
    }

    #[test]
    fn ignores_blobs_without_marker() {
        let roots = game_roots_from_paths(["Games/a/info.json5", "Games/a/rules/preview/icon.png"]);
        assert!(roots.is_empty());
    }

    #[test]
    fn parse_info_full() {
        let text = r#"{
            preview: "preview.html",
            icon: "icon.png",
            info: {
                name: { EN: "Poker", RU: "Покер" },
                tags: ["classic", "cards"]
            }
        }"#;
        let info = parse_info_json5(text).unwrap();
        assert_eq!(info.preview.as_deref(), Some("preview.html"));
        assert_eq!(info.icon.as_deref(), Some("icon.png"));
        assert_eq!(info.name_for_locale("EN"), Some("Poker"));
        assert_eq!(info.name_for_locale("RU"), Some("Покер"));
        assert_eq!(info.tags, vec!["classic", "cards"]);
    }

    #[test]
    fn parse_info_missing_name_icon_preview() {
        let info = parse_info_json5("{ info: { tags: \"nope\" } }").unwrap();
        assert!(info.preview.is_none());
        assert!(info.icon.is_none());
        assert!(info.names.is_empty());
        assert!(info.tags.is_empty());
        assert!(info.name_for_locale("EN").is_none());
    }

    #[test]
    fn parse_info_broken_is_none() {
        assert!(parse_info_json5("not json5 {").is_none());
        assert!(parse_info_json5("[1, 2]").is_none());
    }

    #[test]
    fn search_matches_name_tag_case_and_empty() {
        let mut info = GameInfo::default();
        info.names.insert("EN".into(), "Poker".into());
        info.names.insert("RU".into(), "Покер".into());
        info.tags.push("Classic".into());
        assert!(info_matches_query(&info, ""));
        assert!(info_matches_query(&info, "  "));
        assert!(info_matches_query(&info, "poker"));
        assert!(info_matches_query(&info, "ПОКЕР"));
        assert!(info_matches_query(&info, "clas"));
        assert!(!info_matches_query(&info, "bridge"));
    }

    #[test]
    fn unique_name_still_available_for_install() {
        use deck_gen_wasm_fs::unique_name;
        let taken = |name: &str| name == "Poker";
        assert_eq!(unique_name("Poker", taken), "Poker-1");
    }
}
