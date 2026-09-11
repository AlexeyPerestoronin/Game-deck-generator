//! Shared PDF types, A4 imposition, and the HTML→PDF engine trait.
//!
//! Layout and impose run on every target. Chromium lives in `prepare_pdf_host`
//! (CLI wrapper [`HostPdfEngine`]); the browser raster path is `prepare_pdf_web`.

mod images;
mod impose;
mod layout;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
mod host;

use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::fs::FileSystem;

pub use images::{pdf_from_jpeg_pages, JpegPage};
pub use impose::impose_duplex_bytes;
pub use layout::{mm_to_pt, CardSize, Duplex, SheetLayout, Slot};

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub use host::HostPdfEngine;

/// Convert self-contained card HTML into a multi-page PDF of card-sized pages.
pub trait PdfEngineGenerator {
    fn html_to_pdf(&self, html: &str, card: CardSize) -> impl Future<Output = Result<Vec<u8>>>;
}

/// Paths written by [`prepare_pdf_named`].
pub struct PdfArtifacts {
    /// `cards.len()` after row expansion.
    pub card_count: usize,
    /// Preview page.
    pub preview: PathBuf,
    /// Face sheet HTML.
    pub face_html: PathBuf,
    /// Back sheet HTML.
    pub back_html: PathBuf,
    /// Face PDF.
    pub face_pdf: PathBuf,
    /// Back PDF.
    pub back_pdf: PathBuf,
    /// Duplex sheet PDF in the deck output dir.
    pub duplex: PathBuf,
    /// Copy under the game duplex folder, named `{deck}.pdf`.
    pub collected_duplex: PathBuf,
}

/// Render HTML then PDF for every deck visible through `fs`.
pub async fn prepare_pdf<F, E>(fs: Arc<F>, engine: &E) -> Result<usize>
where
    F: FileSystem + ?Sized + 'static,
    E: PdfEngineGenerator,
{
    Ok(prepare_pdf_named(fs, engine, None, None).await?.len())
}

/// Same as [`prepare_pdf`], optionally restricted to a deck name/prefix and duplex mode.
pub async fn prepare_pdf_named<F, E>(
    fs: Arc<F>,
    engine: &E,
    name: Option<&str>,
    duplex_override: Option<&str>,
) -> Result<Vec<(String, PdfArtifacts)>>
where
    F: FileSystem + ?Sized + 'static,
    E: PdfEngineGenerator,
{
    let loaded = crate::conf::load(fs.as_ref())?;
    let decks = crate::catalog::find_decks(fs.as_ref(), &loaded, name)?;
    let mut out = Vec::new();
    for deck in decks {
        let game = loaded.game_for_deck_name(&deck.name)?;
        let duplex_label = duplex_override.unwrap_or(&game.print.default_duplex);
        let duplex = Duplex::parse(duplex_label).map_err(Error::msg)?;
        fs.create_dir_all(&game.duplex)?;
        let html = crate::render::prepare_html(&fs, &loaded, &deck)?;
        let card = CardSize {
            width_mm: deck.card_width_mm(),
            height_mm: deck.card_height_mm(),
        };
        let face_html = fs.read_to_string(&html.face_html)?;
        let back_html = fs.read_to_string(&html.back_html)?;
        let face_pdf_bytes = engine.html_to_pdf(&face_html, card).await?;
        let back_pdf_bytes = engine.html_to_pdf(&back_html, card).await?;
        let duplex_bytes = impose::impose_duplex_bytes(
            &face_pdf_bytes,
            &back_pdf_bytes,
            card,
            duplex,
            SheetLayout::from_print(&game.print),
        )?;
        let out_dir = deck.output_dir(&loaded)?;
        let face_pdf = out_dir.join(&game.output.face_pdf);
        let back_pdf = out_dir.join(&game.output.back_pdf);
        let duplex_pdf = out_dir.join(&game.output.duplex_pdf);
        fs.write_bytes(&face_pdf, &face_pdf_bytes)?;
        fs.write_bytes(&back_pdf, &back_pdf_bytes)?;
        fs.write_bytes(&duplex_pdf, &duplex_bytes)?;
        let collected_duplex = game.duplex.join(format!("{}.pdf", deck.name));
        fs.write_bytes(&collected_duplex, &duplex_bytes)?;
        out.push((
            deck.name,
            PdfArtifacts {
                card_count: html.card_count,
                preview: html.preview,
                face_html: html.face_html,
                back_html: html.back_html,
                face_pdf,
                back_pdf,
                duplex: duplex_pdf,
                collected_duplex,
            },
        ));
    }
    Ok(out)
}
