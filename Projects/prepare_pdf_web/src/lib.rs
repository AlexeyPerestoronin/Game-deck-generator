//! Browser `PdfEngineGenerator` used by `deck_gen_wasm`.
//!
//! Also provides WebPngEngine (CardPngGenerator) for per-card HTML → PNG raster
//! using the same DOM-paint approach (canvas.toBlob('image/png')).
//!
//! Card HTML is laid out in a hidden iframe (so CSS and fit-header scripts run),
//! painted to a canvas from the live DOM, encoded as JPEG/PNG. Painting from the
//! DOM keeps the canvas origin-clean in Chromium.

use std::future::Future;

use deck_gen::pdf_engine::{pdf_from_jpeg_pages, CardSize, JpegPage, PdfEngineGenerator};
use deck_gen::{CardPngGenerator, Error, Result};
use js_sys::{Reflect, Uint8Array};
use progress_viewer::ProgressHandler;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

const PRINT_DPI: f64 = 300.0;

/// Stateless engine; all work happens in the page DOM.
#[derive(Clone, Copy, Debug, Default)]
pub struct WebPdfEngine;

impl PdfEngineGenerator for WebPdfEngine {
    fn html_to_pdf(&self, html: &str, card: CardSize) -> impl Future<Output = Result<Vec<u8>>> {
        html_to_pdf(html.to_string(), card, &progress_viewer::NoopProgress)
    }
}

/// Stateless PNG raster engine; uses same iframe+canvas paint path as the PDF one.
#[derive(Clone, Copy, Debug, Default)]
pub struct WebPngEngine;

impl CardPngGenerator for WebPngEngine {
    fn html_to_png(&self, html: &str, card: CardSize) -> impl Future<Output = Result<Vec<u8>>> {
        html_to_png(html.to_string(), card.width_mm, card.height_mm, &progress_viewer::NoopProgress)
    }
}

async fn html_to_png(html: String, width_mm: f64, height_mm: f64, progress: &impl ProgressHandler) -> Result<Vec<u8>> {
    progress.set(0.0);
    let promise = html_to_png_js(&html, width_mm, height_mm);
    progress.set(30.0);
    let value = JsFuture::from(promise)
        .await
        .map_err(|err| Error::msg(js_error(err)))?;
    let bytes = Uint8Array::new(&value).to_vec();
    if bytes.is_empty() {
        return Err(Error::msg("browser produced empty PNG"));
    }
    progress.set(100.0);
    Ok(bytes)
}

async fn html_to_pdf(html: String, card: CardSize, progress: &impl ProgressHandler) -> Result<Vec<u8>> {
    progress.set(0.0);
    let promise = html_to_jpeg_pages(&html, card.width_mm, card.height_mm, PRINT_DPI);
    progress.set(30.0);
    let value = JsFuture::from(promise)
        .await
        .map_err(|err| Error::msg(js_error(err)))?;
    let pages = parse_pages(&value)?;
    let out = pdf_from_jpeg_pages(&pages, card);
    progress.set(100.0);
    out
}

fn parse_pages(value: &JsValue) -> Result<Vec<JpegPage>> {
    let array = js_sys::Array::from(value);
    let mut pages = Vec::with_capacity(array.length() as usize);
    for item in array.iter() {
        let data = Reflect::get(&item, &JsValue::from_str("data"))
            .map_err(|err| Error::msg(js_error(err)))?;
        let width = Reflect::get(&item, &JsValue::from_str("width"))
            .map_err(|err| Error::msg(js_error(err)))?
            .as_f64()
            .ok_or_else(|| Error::msg("JPEG page width is not a number"))?;
        let height = Reflect::get(&item, &JsValue::from_str("height"))
            .map_err(|err| Error::msg(js_error(err)))?
            .as_f64()
            .ok_or_else(|| Error::msg("JPEG page height is not a number"))?;
        let bytes = Uint8Array::new(&data).to_vec();
        pages.push(JpegPage {
            jpeg: bytes,
            width_px: width as u32,
            height_px: height as u32,
        });
    }
    if pages.is_empty() {
        return Err(Error::msg("browser produced no card pages"));
    }
    Ok(pages)
}

fn js_error(err: JsValue) -> String {
    err.as_string()
        .or_else(|| {
            Reflect::get(&err, &JsValue::from_str("message"))
                .ok()
                .and_then(|value| value.as_string())
        })
        .unwrap_or_else(|| "browser PDF render failed".into())
}

#[wasm_bindgen(module = "/src/html_to_jpeg_pages.js")]
extern "C" {
    #[wasm_bindgen(js_name = htmlToJpegPages)]
    fn html_to_jpeg_pages(html: &str, width_mm: f64, height_mm: f64, dpi: f64) -> js_sys::Promise;

    #[wasm_bindgen(js_name = htmlToPng)]
    fn html_to_png_js(html: &str, width_mm: f64, height_mm: f64) -> js_sys::Promise;
}
