//! Host-only PDF pipeline: Chromium print + A4 duplex imposition.
//!
//! Callers pass layout and Chrome search rules. This crate does not read
//! `conf.json5` and must not be linked into a wasm build of `deck_gen`.

mod chrome;
mod error;
mod impose;
mod layout;

use std::path::PathBuf;

pub use chrome::{Chrome, ChromeLocator};
pub use error::{Error, Result};
pub use layout::{CardSize, Duplex, SheetLayout};

use impose::impose_duplex;

#[derive(Clone, Debug)]
pub struct PdfJob {
    pub card: CardSize,
    pub output_dir: PathBuf,
    pub face_html: PathBuf,
    pub back_html: PathBuf,
    pub face_pdf_name: String,
    pub back_pdf_name: String,
    pub duplex_pdf_name: String,
    pub sheet: SheetLayout,
}

#[derive(Clone, Debug)]
pub struct PdfArtifacts {
    pub face_pdf: PathBuf,
    pub back_pdf: PathBuf,
    pub duplex: PathBuf,
}

pub fn prepare_pdf_with(chrome: &Chrome, job: &PdfJob, duplex: Duplex) -> Result<PdfArtifacts> {
    std::fs::create_dir_all(&job.output_dir)?;
    let face_pdf = job.output_dir.join(&job.face_pdf_name);
    let back_pdf = job.output_dir.join(&job.back_pdf_name);
    chrome.html_file_to_pdf(&job.face_html, &face_pdf, job.card)?;
    chrome.html_file_to_pdf(&job.back_html, &back_pdf, job.card)?;
    let duplex_pdf = job.output_dir.join(&job.duplex_pdf_name);
    impose_duplex(
        &face_pdf,
        &back_pdf,
        &duplex_pdf,
        job.card,
        duplex,
        job.sheet,
    )?;
    Ok(PdfArtifacts {
        face_pdf,
        back_pdf,
        duplex: duplex_pdf,
    })
}

impl Chrome {
    pub fn render_job(&self, job: &PdfJob, duplex: Duplex) -> Result<PdfArtifacts> {
        prepare_pdf_with(self, job, duplex)
    }
}
