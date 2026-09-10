//! HTML file → PDF via a local Chromium (CDP).

use std::fs;
use std::path::{Path, PathBuf};

use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::{Browser, LaunchOptions};

use crate::error::{Error, Result};
use crate::layout::CardSize;

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

    pub fn html_file_to_pdf(&self, html: &Path, pdf: &Path, card: CardSize) -> Result<()> {
        if !html.is_file() {
            return Err(Error::file(html, "HTML file does not exist"));
        }
        let tab = self.browser.new_tab()?;
        tab.navigate_to(&path_to_file_url(html)?)?;
        tab.wait_until_navigated()?;
        let _ = tab.evaluate("window.__fitHeaderNames || document.fonts.ready", true);
        let bytes = tab.print_to_pdf(Some(pdf_options(card)))?;
        if let Some(parent) = pdf.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(pdf, bytes)?;
        Ok(())
    }
}

fn pdf_options(card: CardSize) -> PrintToPdfOptions {
    PrintToPdfOptions {
        print_background: Some(true),
        prefer_css_page_size: Some(true),
        paper_width: Some(card.width_mm / 25.4),
        paper_height: Some(card.height_mm / 25.4),
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
