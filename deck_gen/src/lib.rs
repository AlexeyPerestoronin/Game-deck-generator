//! JSON5 deck data plus Jinja/SCSS views, rendered to card HTML.
//!
//! Native builds with `--features cli` also print PDFs through Chromium
//! (`pdf_engine::HostPdfEngine`). WASM uses `prepare_pdf_web` as the engine.
//!
//! PNG generation (per-card) is independent of PDF: it consumes the per-card
//! HTML side artifacts produced by render and rasterizes them (native via CDP
//! screenshot in prepare_pdf_host; WASM via canvas in prepare_pdf_web).
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

use std::future::Future;
use std::path::PathBuf;
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
/// The `name` (when Some) is a game id (selects all its decks), a bare deck name,
/// or "game.deckname" (selects specific). Each item is `(deck_name, artifacts)`.
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

/// Raster a self-contained single-card HTML document to PNG bytes.
///
/// Implementations exist for native Chrome (CDP screenshot) and WASM (canvas paint).
pub trait CardPngGenerator {
    /// Produce PNG bytes for the card at the provided CSS dimensions.
    fn html_to_png(&self, html: &str, card: pdf_engine::CardSize) -> impl Future<Output = Result<Vec<u8>>>;
}

/// Paths written by [`prepare_png_named`].
pub struct PngArtifacts {
    /// `cards.len()` after row expansion.
    pub card_count: usize,
    /// Preview page.
    pub preview: PathBuf,
    /// Face sheet HTML.
    pub face_html: PathBuf,
    /// Back sheet HTML.
    pub back_html: PathBuf,
    /// Directory with per-card PNGs (`card-N-face.png`, `card-N-back.png`).
    pub png_dir: PathBuf,
}

/// Render per-card PNGs for every deck visible through `fs` (requires prior or
/// internal HTML preparation for the per-card HTML files).
pub async fn prepare_png<F, E>(fs: Arc<F>, engine: &E) -> Result<usize>
where
    F: FileSystem + ?Sized + 'static,
    E: CardPngGenerator,
{
    Ok(prepare_png_named(fs, engine, None).await?.len())
}

/// Same as [`prepare_png`], optionally restricted to deck name/prefix.
///
/// Each returned item is `(deck_label, artifacts)`. PNGs are written under
/// the deck's output dir / cards/png / using the per-card HTML files as input.
pub async fn prepare_png_named<F, E>(
    fs: Arc<F>,
    engine: &E,
    name: Option<&str>,
) -> Result<Vec<(String, PngArtifacts)>>
where
    F: FileSystem + ?Sized + 'static,
    E: CardPngGenerator,
{
    let loaded = crate::conf::load(fs.as_ref())?;
    let decks = crate::catalog::find_decks(fs.as_ref(), &loaded, name)?;
    let mut out = Vec::new();
    for deck in decks {
        // Per-card HTMLs are created as side-effect of prepare_html (see render.rs).
        let html = crate::render::prepare_html(&fs, &loaded, &deck)?;
        let card = pdf_engine::CardSize {
            width_mm: deck.card_width_mm(),
            height_mm: deck.card_height_mm(),
        };
        let out_dir = deck.output_dir(&loaded)?;
        let png_dir = out_dir.join("cards").join("png");
        fs.create_dir_all(&png_dir)?;
        let cards_html_dir = out_dir.join("cards").join("html");
        for i in 0..html.card_count {
            let n = i + 1;
            let face_html_path = cards_html_dir.join(format!("card-{n}-face.html"));
            let back_html_path = cards_html_dir.join(format!("card-{n}-back.html"));
            let face_html = fs.read_to_string(&face_html_path)?;
            let back_html = fs.read_to_string(&back_html_path)?;
            let face_png = engine.html_to_png(&face_html, card).await?;
            let back_png = engine.html_to_png(&back_html, card).await?;
            fs.write_bytes(&png_dir.join(format!("card-{n}-face.png")), &face_png)?;
            fs.write_bytes(&png_dir.join(format!("card-{n}-back.png")), &back_png)?;
        }
        out.push((
            deck.name,
            PngArtifacts {
                card_count: html.card_count,
                preview: html.preview,
                face_html: html.face_html,
                back_html: html.back_html,
                png_dir,
            },
        ));
    }
    Ok(out)
}

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
impl CardPngGenerator for prepare_pdf_host::Chrome {
    fn html_to_png(&self, html: &str, card: pdf_engine::CardSize) -> impl Future<Output = Result<Vec<u8>>> {
        let out = self
            .html_to_png_bytes(html, card.width_mm, card.height_mm)
            .map_err(|err| Error::msg(err.to_string()));
        std::future::ready(out)
    }
}
