//! Discover decks under each game's `decks/` folder.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::conf::{conf, GamePaths};
use crate::error::{Error, Result};
use crate::load::DataManager;
use crate::model::Deck;

#[derive(Clone)]
struct LocatedDeck {
    path: PathBuf,
    name: String,
}

pub fn list_deck_names() -> Result<Vec<String>> {
    Ok(locate_all()?.into_iter().map(|item| item.name).collect())
}

pub fn matching_names(query: Option<&str>) -> Result<Vec<String>> {
    Ok(matching_located(query)?.into_iter().map(|item| item.name).collect())
}

pub fn find_decks(query: Option<&str>) -> Result<Vec<Deck>> {
    matching_located(query)?
        .iter()
        .map(|item| load_deck(&item.path, &item.name))
        .collect()
}

fn matching_located(query: Option<&str>) -> Result<Vec<LocatedDeck>> {
    let located = locate_all()?;
    if located.is_empty() {
        return Err(Error::msg("No deck data files found under configured games"));
    }
    let Some(needle) = query else {
        return Ok(located);
    };
    let default_game = conf()?.default_game.clone();
    let matched: Vec<LocatedDeck> = located
        .iter()
        .filter(|item| name_matches_query(&item.name, needle, &default_game))
        .cloned()
        .collect();
    if matched.is_empty() {
        let known = located.into_iter().map(|item| item.name).collect::<Vec<_>>().join(", ");
        return Err(Error::msg(format!("Unknown deck {needle:?}. Known: {known}")));
    }
    Ok(matched)
}

fn locate_all() -> Result<Vec<LocatedDeck>> {
    let loaded = conf()?;
    let mut by_dir: BTreeMap<PathBuf, LocatedDeck> = BTreeMap::new();
    for game in loaded.games.values() {
        if game.decks.is_dir() {
            collect_data_files(game, &game.decks, &mut by_dir)?;
        }
    }
    Ok(by_dir.into_values().collect())
}

fn collect_data_files(
    game: &GamePaths,
    dir: &Path,
    by_dir: &mut BTreeMap<PathBuf, LocatedDeck>,
) -> Result<()> {
    for entry in dir.read_dir()? {
        let path = entry?.path();
        if path.is_dir() {
            collect_data_files(game, &path, by_dir)?;
        }
    }
    let data = dir.join("data.json5");
    if data.is_file() {
        let name = dotted_name(game, &data)?;
        by_dir.insert(dir.to_path_buf(), LocatedDeck { path: data, name });
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

fn load_deck(data_file: &Path, expected_name: &str) -> Result<Deck> {
    let deck = Deck::from_manager(&DataManager::new(data_file)?)?;
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
