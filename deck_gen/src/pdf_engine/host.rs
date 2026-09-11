//! Native `PdfEngineGenerator`: Chromium print via `prepare_pdf_host`.

use std::future::Future;

use prepare_pdf_host::Chrome;

use super::{CardSize, PdfEngineGenerator};
use crate::error::{Error, Result};

/// Wraps a launched Chrome so `prepare_pdf` does not depend on the host crate's types.
pub struct HostPdfEngine {
    chrome: Chrome,
}

impl HostPdfEngine {
    pub fn new(chrome: Chrome) -> Self {
        Self { chrome }
    }
}

impl PdfEngineGenerator for HostPdfEngine {
    fn html_to_pdf(&self, html: &str, card: CardSize) -> impl Future<Output = Result<Vec<u8>>> {
        let out = self
            .chrome
            .html_to_pdf_bytes(html, card.width_mm, card.height_mm)
            .map_err(|err| Error::msg(err.to_string()));
        std::future::ready(out)
    }
}
