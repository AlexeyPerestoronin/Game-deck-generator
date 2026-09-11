//! Jinja + SCSS rendering of face / back / preview HTML.
//!
//! Templates are loaded from the deck directory first, then the game `views/`.
//! `.scss` files are run through MiniJinja (so they can use the same globals)
//! and then `grass`. That needs two environments: a “raw” loader without SCSS
//! compilation, and the public loader that compiles `.scss` using the raw env.
//! The MiniJinja loader is `'static`, so the [`FileSystem`] is held in an `Arc`.

use std::path::PathBuf;
use std::sync::Arc;

use minijinja::value::Value as JinjaValue;
use minijinja::{context, AutoEscape, Environment};
use serde_json::Value;

use crate::conf::{views_for_deck_name, Conf};
use crate::error::Result;
use crate::fs::FileSystem;
use crate::model::Deck;

/// Paths written by [`prepare_html`] and the number of cards in the deck.
pub struct HtmlArtifacts {
    /// Preview page (`preview.html` by default).
    pub preview: PathBuf,
    /// Face sheet HTML.
    pub face_html: PathBuf,
    /// Back sheet HTML.
    pub back_html: PathBuf,
    /// `cards.len()` after row expansion.
    pub card_count: usize,
}

/// Render face, back, and preview HTML for `deck` into its output directory.
pub fn prepare_html<F>(
    fs: &Arc<F>,
    loaded: &Conf,
    deck: &Deck,
) -> Result<HtmlArtifacts>
where
    F: FileSystem + ?Sized + 'static,
{
    render_html(fs, loaded, deck, &deck.cards())
}

fn render_html<F>(
    fs: &Arc<F>,
    loaded: &Conf,
    deck: &Deck,
    cards: &[Value],
) -> Result<HtmlArtifacts>
where
    F: FileSystem + ?Sized + 'static,
{
    let game = loaded.game_for_deck_name(&deck.name)?;
    let env = jinja_env(fs.clone(), loaded, deck)?;
    let out = deck.output_dir(loaded)?;
    fs.create_dir_all(&out)?;

    let face_template = deck.template_for("face")?;
    let back_template = deck.template_for("back")?;
    let ctx = template_context(deck, cards);

    let face_path = out.join(&game.output.face_html);
    let back_path = out.join(&game.output.back_html);
    let preview_path = out.join(&game.output.preview_html);
    fs.write(&face_path, &env.get_template(&face_template)?.render(&ctx)?)?;
    fs.write(&back_path, &env.get_template(&back_template)?.render(&ctx)?)?;
    fs.write(
        &preview_path,
        &env.get_template(&game.output.preview_html)?.render(&ctx)?,
    )?;

    Ok(HtmlArtifacts {
        preview: preview_path,
        face_html: face_path,
        back_html: back_path,
        card_count: cards.len(),
    })
}

fn template_context(deck: &Deck, cards: &[Value]) -> minijinja::value::Value {
    let deck_obj = JinjaValue::from_serialize(&deck.fields);
    let cards_obj = JinjaValue::from_serialize(cards);
    context! {
        deck => deck_obj,
        cards => cards_obj,
        game_name => deck.game_name(),
        card_width_mm => deck.card_width_mm(),
        card_height_mm => deck.card_height_mm(),
    }
}

fn jinja_env<F>(
    fs: Arc<F>,
    loaded: &Conf,
    deck: &Deck,
) -> Result<Environment<'static>>
where
    F: FileSystem + ?Sized + 'static,
{
    let search = Arc::new(vec![
        deck.directory.clone(),
        views_for_deck_name(loaded, &deck.name)?,
    ]);
    let mut raw_env = Environment::new();
    configure_env(&mut raw_env, deck);
    attach_loader(&mut raw_env, fs.clone(), search.clone(), None);

    let mut env = Environment::new();
    configure_env(&mut env, deck);
    attach_loader(&mut env, fs, search, Some(Arc::new(raw_env)));
    Ok(env)
}

fn configure_env(env: &mut Environment<'static>, deck: &Deck) {
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);
    env.set_auto_escape_callback(|name| {
        if name.ends_with(".html") || name.ends_with(".xml") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });
    env.add_global("game_name", deck.game_name());
    env.add_global("card_width_mm", deck.card_width_mm());
    env.add_global("card_height_mm", deck.card_height_mm());
}

fn attach_loader<F>(
    env: &mut Environment<'static>,
    fs: Arc<F>,
    search: Arc<Vec<PathBuf>>,
    scss_env: Option<Arc<Environment<'static>>>,
) where
    F: FileSystem + ?Sized + 'static,
{
    env.set_loader(move |name| {
        let Some(raw) = read_template(name, fs.as_ref(), &search) else {
            return Ok(None);
        };
        if let Some(scss_env) = &scss_env {
            if name.ends_with(".scss") {
                let rendered = scss_env.render_str(&raw, context! {})?;
                let css = grass::from_string(rendered, &grass::Options::default()).map_err(|err| {
                    minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, err.to_string())
                })?;
                return Ok(Some(css));
            }
        }
        Ok(Some(raw))
    });
}

fn read_template<F>(name: &str, fs: &F, search: &[PathBuf]) -> Option<String>
where
    F: FileSystem + ?Sized,
{
    for dir in search {
        let path = dir.join(name);
        if let Ok(raw) = fs.read_to_string(&path) {
            return Some(raw);
        }
    }
    None
}
