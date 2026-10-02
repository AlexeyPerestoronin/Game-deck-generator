//! Local game detection and scoped VFS isolate/merge for Prepare HTML/PDF.

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::{file_name, path_is_or_under, Vfs};
use deck_gen_wasm_locale as locale;
use deck_gen_wasm_locale::keys;
use deck_gen_wasm_template::{game_roots_from_paths, parse_info_json5, GameInfo};

/// One game found in persist-VFS (a folder that contains `game.json5`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalGame {
    /// Game root path in persist-VFS (e.g. `games/poker`).
    pub root: String,
    /// Parsed `info.json5`, or defaults when missing/broken.
    pub info: GameInfo,
    /// True when `info.json5` existed and parsed.
    pub has_info: bool,
}

/// Scan persist-VFS for game roots. Missing/broken info uses defaults; games are kept.
pub fn local_games(vfs: &Vfs) -> Vec<LocalGame> {
    let mut paths = Vec::new();
    vfs.visit_entries(|path, _| paths.push(path.to_string()));
    game_roots_from_paths(paths)
        .into_iter()
        .map(|root| {
            let info_path = join_under(&root, conf::catalog::PREVIEW_DIR, conf::catalog::INFO_FILE);
            match vfs.read_file(&info_path) {
                Some(text) => match parse_info_json5(text) {
                    Some(info) => LocalGame {
                        root,
                        info,
                        has_info: true,
                    },
                    None => LocalGame {
                        root,
                        info: GameInfo::default(),
                        has_info: false,
                    },
                },
                None => LocalGame {
                    root,
                    info: GameInfo::default(),
                    has_info: false,
                },
            }
        })
        .collect()
}

fn join_under(root: &str, mid: &str, file: &str) -> String {
    if root.is_empty() {
        format!("{mid}/{file}")
    } else {
        format!("{root}/{mid}/{file}")
    }
}

fn locale_key() -> &'static str {
    match locale::get_active_locale() {
        locale::Locale::En => "EN",
        locale::Locale::Ru => "RU",
    }
}

/// Card title: localized name, else EN, else stub.
pub fn game_card_name(info: &GameInfo) -> String {
    info.name_for_locale(locale_key())
        .map(str::to_string)
        .unwrap_or_else(|| locale::localize(keys::GAMES_NAME_UNDEFINED))
}

/// Prepare-menu label: info name, else stub if info existed, else last path segment.
pub fn game_menu_name(game: &LocalGame) -> String {
    if let Some(name) = game.info.name_for_locale(locale_key()) {
        return name.to_string();
    }
    if game.has_info {
        return locale::localize(keys::GAMES_NAME_UNDEFINED);
    }
    let seg = file_name(&game.root);
    if seg.is_empty() {
        locale::localize(keys::GAMES_NAME_UNDEFINED)
    } else {
        seg.to_string()
    }
}

/// Working VFS with only the selected game trees (no shared `games/conf.json5`).
pub fn isolate_game_trees(src: &Vfs, roots: &[String]) -> Vfs {
    let mut out = Vfs::default();
    src.visit_entries(|path, content| {
        if !roots.iter().any(|root| path_is_or_under(path, root)) {
            return;
        }
        match content {
            Some(bytes) => {
                let _ = out.put_bytes(path, bytes.to_vec());
            }
            None => {
                let _ = out.mkdir(path);
            }
        }
    });
    out
}

/// Overwrite persist files that sit under the selected roots.
pub fn merge_game_trees(dest: &mut Vfs, working: &Vfs, roots: &[String]) {
    working.visit_entries(|path, content| {
        if !roots.iter().any(|root| path_is_or_under(path, root)) {
            return;
        }
        if let Some(bytes) = content {
            let _ = dest.put_bytes(path, bytes.to_vec());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolate_keeps_only_selected_game_trees() {
        let mut src = Vfs::default();
        src.put_file("games/a/game.json5", "a".into()).unwrap();
        src.put_file("games/a/x.txt", "ax".into()).unwrap();
        src.put_file("games/b/game.json5", "b".into()).unwrap();
        src.put_file("games/conf.json5", "conf".into()).unwrap();
        src.put_file("other.txt", "o".into()).unwrap();
        let working = isolate_game_trees(&src, &["games/a".into()]);
        assert!(working.is_file("games/a/game.json5"));
        assert!(working.is_file("games/a/x.txt"));
        assert!(!working.exists("games/b/game.json5"));
        assert!(!working.exists("games/conf.json5"));
        assert!(!working.exists("other.txt"));
    }

    #[test]
    fn merge_writes_back_only_under_selected_roots() {
        let mut persist = Vfs::default();
        persist.put_file("games/a/game.json5", "old".into()).unwrap();
        persist.put_file("games/b/game.json5", "b".into()).unwrap();
        persist.put_file("games/conf.json5", "conf".into()).unwrap();
        let mut working = Vfs::default();
        working.put_file("games/a/game.json5", "new".into()).unwrap();
        working.put_file("games/a/out.html", "html".into()).unwrap();
        working
            .put_file("games/conf.json5", "nope".into())
            .unwrap();
        merge_game_trees(&mut persist, &working, &["games/a".into()]);
        assert_eq!(persist.read_file("games/a/game.json5"), Some("new"));
        assert_eq!(persist.read_file("games/a/out.html"), Some("html"));
        assert_eq!(persist.read_file("games/b/game.json5"), Some("b"));
        assert_eq!(persist.read_file("games/conf.json5"), Some("conf"));
    }

    #[test]
    fn local_games_detects_nested_and_keeps_broken_info() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/game.json5", "{}".into()).unwrap();
        vfs.put_file("games/a/rules/preview/info.json5", "not json".into())
            .unwrap();
        vfs.put_file("games/nested/b/game.json5", "{}".into())
            .unwrap();
        let games = local_games(&vfs);
        let roots: Vec<_> = games.iter().map(|g| g.root.as_str()).collect();
        assert_eq!(roots, vec!["games/a", "games/nested/b"]);
        assert!(!games[0].has_info);
        assert!(!games[1].has_info);
    }
}
