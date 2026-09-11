//! JSON5 deck data plus Jinja/SCSS views, rendered to card HTML.
//!
//! Native builds with `--features cli` also print PDFs through Chromium
//! (`pdf_engine::HostPdfEngine`). WASM uses `prepare_pdf_web` as the engine.
//!
//! Pipeline: load [`conf`] → [`catalog`] matching decks → [`load`] / [`subst`]
//! expand placeholders → [`model::Deck`] + [`card`] rows → [`render`] HTML.

pub mod card;
pub mod catalog;
pub mod conf;
pub mod error;
pub mod fs;
pub mod load;
pub mod model;
pub mod pdf_engine;
pub mod render;
pub mod subst;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod cli;

use std::sync::Arc;

pub use error::{Error, Result};
pub use fs::FileSystem;
pub use model::Deck;
pub use pdf_engine::{prepare_pdf, prepare_pdf_named, PdfEngineGenerator};

/// Render face / back / preview HTML for every deck visible through `fs`.
///
/// Returns the number of decks written. `F` is dispatched statically; pass
/// `Arc<dyn FileSystem>` only when the implementation is chosen at runtime.
pub fn prepare_html<F>(fs: Arc<F>) -> Result<usize>
where
    F: FileSystem + ?Sized + 'static,
{
    Ok(prepare_html_named(fs, None)?.len())
}

/// Same as [`prepare_html`], but optionally restrict to a deck name or prefix.
///
/// Each item is `(deck_name, artifacts)` in catalog order.
pub fn prepare_html_named<F>(
    fs: Arc<F>,
    name: Option<&str>,
) -> Result<Vec<(String, render::HtmlArtifacts)>>
where
    F: FileSystem + ?Sized + 'static,
{
    let loaded = conf::load(fs.as_ref())?;
    let decks = catalog::find_decks(fs.as_ref(), &loaded, name)?;
    let mut out = Vec::new();
    for deck in decks {
        let artifacts = render::prepare_html(&fs, &loaded, &deck)?;
        out.push((deck.name, artifacts));
    }
    Ok(out)
}
