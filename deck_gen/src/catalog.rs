//! Discover decks under each game's `decks/` folder.
//!
//! A deck is a directory that contains `data.json5`. The deck `name` is taken from
//! the `name` field inside `data.json5` (any string, must be unique per game).
//! The folder location no longer dictates the name. Queries: game id selects all
//! decks of the game; "game.deckname" or bare deckname also supported. Discovery
//! walks + BTreeMap for stable order.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::conf::{Conf, GamePaths};
use crate::error::{Error, Result};
use crate::fs::FileSystem;
use crate::load::{peek_deck_name, DataManager};
use crate::model::Deck;

#[derive(Clone)]
struct LocatedDeck {
    path: PathBuf,
    name: String,
    game_id: String,
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

/// Load every matching deck. Deck `name` is taken verbatim from data.json5
/// (game prefix logic removed; name independent of containing folder).
pub fn find_decks<F>(fs: &F, loaded: &Conf, query: Option<&str>) -> Result<Vec<Deck>>
where
    F: FileSystem + ?Sized,
{
    matching_located(fs, loaded, query)?
        .iter()
        .map(|item| load_deck(fs, &item.path, &item.game_id, &item.vars))
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
        .filter(|item| name_matches_query(item, needle))
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
        let name = peek_deck_name(fs, &data)?;
        by_dir.insert(
            dir.to_path_buf(),
            LocatedDeck {
                path: data,
                name,
                game_id: game.id.clone(),
                vars: game.vars.clone(),
            },
        );
    }
    Ok(())
}

fn load_deck<F>(
    fs: &F,
    data_file: &Path,
    game_id: &str,
    vars_dir: &Path,
) -> Result<Deck>
where
    F: FileSystem + ?Sized,
{
    let mut deck = Deck::from_manager(&DataManager::new(fs, data_file, vars_dir.to_path_buf())?)?;
    // name is taken from json as-is (any value allowed; folder no longer constrains it)
    deck.game_id = game_id.to_string();
    Ok(deck)
}

fn name_matches_query(item: &LocatedDeck, query: &str) -> bool {
    // exact on declared deck name
    if item.name == query {
        return true;
    }
    // bare game id selects every deck of that game
    if item.game_id == query {
        return true;
    }
    // support "game.deckname" (and "game.sub.deck" for dotted names) using the declared deck name
    if let Some((g, d)) = query.split_once('.') {
        if item.game_id == g && (item.name == d || item.name.starts_with(&format!("{}.", d))) {
            return true;
        }
    }
    // tolerate old-style full names passed as query
    if let Some((_, d)) = query.split_once('.') {
        if item.name == d || item.name.starts_with(&format!("{}.", d)) {
            return true;
        }
    }
    false
}
