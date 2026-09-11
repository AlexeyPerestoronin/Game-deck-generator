//! Jinja + SCSS rendering of face / back / preview HTML.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use minijinja::value::Value as JinjaValue;
use minijinja::{context, AutoEscape, Environment};
use serde_json::{json, Value};

use crate::conf::{conf, manifest_file_name, preview_template_name, views_for_deck_name};
use crate::error::Result;
use crate::model::Deck;

pub struct HtmlArtifacts {
    pub preview: PathBuf,
    pub face_html: PathBuf,
    pub back_html: PathBuf,
    pub card_count: usize,
}

pub fn prepare_html(deck: &Deck) -> Result<HtmlArtifacts> {
    let cards = deck.cards();
    let artifacts = render_html(deck, &cards)?;
    write_manifest(deck, &artifacts)?;
    Ok(artifacts)
}

fn render_html(deck: &Deck, cards: &[Value]) -> Result<HtmlArtifacts> {
    let loaded = conf()?;
    let game = loaded.game_for_deck_name(&deck.name)?;
    let env = jinja_env(deck)?;
    let out = deck.output_dir()?;
    fs::create_dir_all(&out)?;

    let face_template = deck.template_for("face")?;
    let back_template = deck.template_for("back")?;
    let ctx = template_context(deck, cards);

    let face_path = out.join(&game.output.face_html);
    let back_path = out.join(&game.output.back_html);
    let preview_path = out.join(&game.output.preview_html);
    fs::write(&face_path, env.get_template(&face_template)?.render(&ctx)?)?;
    fs::write(&back_path, env.get_template(&back_template)?.render(&ctx)?)?;
    fs::write(
        &preview_path,
        env.get_template(preview_template_name())?.render(&ctx)?,
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

fn jinja_env(deck: &Deck) -> Result<Environment<'static>> {
    let search = Arc::new(vec![
        deck.directory.clone(),
        views_for_deck_name(&deck.name)?,
    ]);
    let mut raw_env = Environment::new();
    configure_env(&mut raw_env, deck);
    attach_loader(&mut raw_env, search.clone(), None);

    let mut env = Environment::new();
    configure_env(&mut env, deck);
    attach_loader(&mut env, search, Some(Arc::new(raw_env)));
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

fn attach_loader(
    env: &mut Environment<'static>,
    search: Arc<Vec<PathBuf>>,
    scss_env: Option<Arc<Environment<'static>>>,
) {
    env.set_loader(move |name| {
        let Some(raw) = read_template(name, &search) else {
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

fn read_template(name: &str, search: &[PathBuf]) -> Option<String> {
    for dir in search {
        let path = dir.join(name);
        if path.is_file() {
            return fs::read_to_string(path).ok();
        }
    }
    None
}

fn write_manifest(deck: &Deck, artifacts: &HtmlArtifacts) -> Result<()> {
    let out = deck.output_dir()?;
    let payload = json!({
        "name": deck.name,
        "card_width_mm": deck.card_width_mm(),
        "card_height_mm": deck.card_height_mm(),
        "card_count": artifacts.card_count,
        "output_dir": out,
        "preview": artifacts.preview,
        "face_html": artifacts.face_html,
        "back_html": artifacts.back_html,
    });
    fs::write(out.join(manifest_file_name()), serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}
