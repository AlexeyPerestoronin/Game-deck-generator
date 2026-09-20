//! HTML file → PDF via a local Chromium (CDP).

use std::fs;
use std::path::{Path, PathBuf};

use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::{Browser, LaunchOptions};

use crate::error::{Error, Result};
use progress_viewer::ProgressHandler;

mod locate;

pub use locate::ChromeLocator;

use locate::find_chrome;

pub struct Chrome {
    browser: Browser,
}

impl Chrome {
    pub fn launch(locator: &ChromeLocator) -> Result<Self> {
        let executable = find_chrome(locator)?;
        let options = LaunchOptions::default_builder()
            .path(Some(executable))
            .headless(true)
            .build()
            .map_err(|err| Error::msg(err.to_string()))?;
        Ok(Self {
            browser: Browser::new(options)?,
        })
    }

    pub fn html_to_pdf_bytes(&self, html: &str, width_mm: f64, height_mm: f64, progress: &impl ProgressHandler) -> Result<Vec<u8>> {
        progress.set(0.0);
        let html_path = unique_temp("html");
        fs::write(&html_path, html)?;
        let _guard = DeleteOnDrop(html_path.clone());
        self.html_file_to_pdf_bytes(&html_path, width_mm, height_mm, progress)
    }

    pub fn html_file_to_pdf(&self, html: &Path, pdf: &Path, width_mm: f64, height_mm: f64, progress: &impl ProgressHandler) -> Result<()> {
        progress.set(0.0);
        let bytes = self.html_file_to_pdf_bytes(html, width_mm, height_mm, progress)?;
        if let Some(parent) = pdf.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(pdf, bytes)?;
        progress.set(100.0);
        Ok(())
    }

    fn html_file_to_pdf_bytes(&self, html: &Path, width_mm: f64, height_mm: f64, progress: &impl ProgressHandler) -> Result<Vec<u8>> {
        if !html.is_file() {
            return Err(Error::file(html, "HTML file does not exist"));
        }
        progress.set(10.0);
        let tab = self.browser.new_tab()?;
        tab.navigate_to(&path_to_file_url(html)?)?;
        tab.wait_until_navigated()?;
        progress.set(50.0);
        let _ = tab.evaluate("window.__fitHeaderNames || document.fonts.ready", true);
        let bytes = tab.print_to_pdf(Some(pdf_options(width_mm, height_mm)))?;
        progress.set(100.0);
        Ok(bytes)
    }

    /// Render a per-card HTML file (standalone, one card) to PNG bytes via CDP screenshot.
    /// Clips to the rendered card element for exact card-sized output.
    pub fn html_to_png_bytes(&self, html: &str, width_mm: f64, height_mm: f64, progress: &impl ProgressHandler) -> Result<Vec<u8>> {
        progress.set(0.0);
        let html_path = unique_temp("html");
        fs::write(&html_path, html)?;
        let _guard = DeleteOnDrop(html_path.clone());
        self.html_file_to_png_bytes(&html_path, width_mm, height_mm, progress)
    }

    fn html_file_to_png_bytes(&self, html: &Path, _width_mm: f64, _height_mm: f64, progress: &impl ProgressHandler) -> Result<Vec<u8>> {
        if !html.is_file() {
            return Err(Error::file(html, "HTML file does not exist"));
        }
        progress.set(20.0);
        let tab = self.browser.new_tab()?;
        tab.navigate_to(&path_to_file_url(html)?)?;
        tab.wait_until_navigated()?;
        progress.set(50.0);
        let _ = tab.evaluate("window.__fitHeaderNames || document.fonts.ready", true);
        // tiny settle time for layout/scripts
        std::thread::sleep(std::time::Duration::from_millis(50));
        let el = tab.wait_for_element("article.card, article.card-back")?;
        let bytes = el.capture_screenshot(CaptureScreenshotFormatOption::Png)?;
        progress.set(100.0);
        Ok(bytes)
    }
}

fn pdf_options(width_mm: f64, height_mm: f64) -> PrintToPdfOptions {
    PrintToPdfOptions {
        print_background: Some(true),
        prefer_css_page_size: Some(true),
        paper_width: Some(width_mm / 25.4),
        paper_height: Some(height_mm / 25.4),
        margin_top: Some(0.0),
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        display_header_footer: Some(false),
        ..PrintToPdfOptions::default()
    }
}

fn path_to_file_url(path: &Path) -> Result<String> {
    let abs = strip_verbatim(path.canonicalize()?);
    let mut utf = abs.to_string_lossy().replace('\\', "/");
    if !utf.starts_with('/') {
        utf.insert(0, '/');
    }
    Ok(format!("file://{utf}"))
}

fn strip_verbatim(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path
    }
}

fn unique_temp(ext: &str) -> PathBuf {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("deck_gen_{n}.{ext}"))
}

struct DeleteOnDrop(PathBuf);

impl Drop for DeleteOnDrop {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
