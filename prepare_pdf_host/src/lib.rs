//! Host-only HTML→PDF via Chromium (CDP).
//!
//! Layout, duplex imposition, and the `PdfEngineGenerator` trait live in
//! `deck_gen::pdf_engine`. This crate must not depend on `deck_gen`.

mod chrome;
mod error;

pub use chrome::{Chrome, ChromeLocator};
pub use error::{Error, Result};
