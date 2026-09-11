//! Discover decks under each game's `decks/` folder.
//!
//! A deck is a directory that contains `data.json5`. The public name is
//! `{game_id}.{relative.dotted.path}` and must match the `name` field inside
//! the file. Queries accept a full name, a name relative to the default game,
//! or a prefix of either. Discovery is a walk + `BTreeMap` so output order is
//! stable across filesystems.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::conf::{Conf, GamePaths};
use crate::error::{Error, Result};
use crate::fs::FileSystem;
use crate::load::DataManager;
use crate::model::Deck;

#[derive(Clone)]
struct LocatedDeck {
    path: PathBuf,
    name: String,
    vars: PathBuf,
}

/// Names of decks that match `query` (or every deck if `query` is `None`).
pub fn matching_names<F>(fs: &F, loaded: &Conf, query: Option<&str>) -> Result<Vec<String>>
where
    F: FileSystem + ?Sized,
{
    Ok(matching_located(fs, loaded, query)?
        .into_iter()
        .map(|item| item.name)
        .collect())
}

/// Load every matching deck, checking that `data.json5` `name` equals the path.
pub fn find_decks<F>(fs: &F, loaded: &Conf, query: Option<&str>) -> Result<Vec<Deck>>
where
    F: FileSystem + ?Sized,
{
    matching_located(fs, loaded, query)?
        .iter()
        .map(|item| load_deck(fs, &item.path, &item.name, &item.vars))
        .collect()
}

fn matching_located<F>(
    fs: &F,
    loaded: &Conf,
    query: Option<&str>,
) -> Result<Vec<LocatedDeck>>
where
    F: FileSystem + ?Sized,
{
    let located = locate_all(fs, loaded)?;
    if located.is_empty() {
        return Err(Error::msg("No deck data files found under configured games"));
    }
    let Some(needle) = query else {
        return Ok(located);
    };
    let matched: Vec<LocatedDeck> = located
        .iter()
        .filter(|item| name_matches_query(&item.name, needle, &loaded.default_game))
        .cloned()
        .collect();
    if matched.is_empty() {
        let known = located.into_iter().map(|item| item.name).collect::<Vec<_>>().join(", ");
        return Err(Error::msg(format!("Unknown deck {needle:?}. Known: {known}")));
    }
    Ok(matched)
}

fn locate_all<F>(fs: &F, loaded: &Conf) -> Result<Vec<LocatedDeck>>
where
    F: FileSystem + ?Sized,
{
    let mut by_dir: BTreeMap<PathBuf, LocatedDeck> = BTreeMap::new();
    for game in loaded.games.values() {
        if fs.is_dir(&game.decks) {
            collect_data_files(fs, game, &game.decks, &mut by_dir)?;
        }
    }
    Ok(by_dir.into_values().collect())
}

fn collect_data_files<F>(
    fs: &F,
    game: &GamePaths,
    dir: &Path,
    by_dir: &mut BTreeMap<PathBuf, LocatedDeck>,
) -> Result<()>
where
    F: FileSystem + ?Sized,
{
    for path in fs.read_dir(dir)? {
        if fs.is_dir(&path) {
            collect_data_files(fs, game, &path, by_dir)?;
        }
    }
    let data = dir.join("data.json5");
    if fs.is_file(&data) {
        let name = dotted_name(game, &data)?;
        by_dir.insert(
            dir.to_path_buf(),
            LocatedDeck {
                path: data,
                name,
                vars: game.vars.clone(),
            },
        );
    }
    Ok(())
}

fn dotted_name(game: &GamePaths, data_file: &Path) -> Result<String> {
    let parent = data_file
        .parent()
        .ok_or_else(|| Error::file(data_file, "has no parent"))?;
    let relative = parent.strip_prefix(&game.decks).map_err(|_| {
        Error::file(data_file, "is not under the game decks directory")
    })?;
    let rest = relative
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join(".");
    Ok(format!("{}.{}", game.id, rest))
}

fn load_deck<F>(
    fs: &F,
    data_file: &Path,
    expected_name: &str,
    vars_dir: &Path,
) -> Result<Deck>
where
    F: FileSystem + ?Sized,
{
    let deck = Deck::from_manager(&DataManager::new(fs, data_file, vars_dir.to_path_buf())?)?;
    if deck.name != expected_name {
        return Err(Error::file(
            data_file,
            format!(
                "name {:?} must match relative path {expected_name:?}",
                deck.name
            ),
        ));
    }
    Ok(deck)
}

fn name_matches_query(full: &str, query: &str, default_game: &str) -> bool {
    let with_default = format!("{default_game}.{query}");
    full == query
        || full == with_default
        || full.starts_with(&format!("{query}."))
        || full.starts_with(&format!("{with_default}."))
}
