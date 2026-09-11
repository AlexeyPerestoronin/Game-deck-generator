pub mod card;
pub mod catalog;
pub mod conf;
pub mod error;
pub mod fs;
pub mod load;
pub mod model;
pub mod render;
pub mod subst;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod cli;

use std::sync::Arc;

pub use error::{Error, Result};
pub use fs::FileSystem;
pub use model::Deck;

/// Render face / back / preview HTML for every deck visible through `fs`.
pub fn prepare_html(fs: Arc<dyn FileSystem>) -> Result<usize> {
    Ok(prepare_html_named(fs, None)?.len())
}

pub fn prepare_html_named(
    fs: Arc<dyn FileSystem>,
    name: Option<&str>,
) -> Result<Vec<(String, render::HtmlArtifacts)>> {
    let loaded = conf::load(fs.as_ref())?;
    let decks = catalog::find_decks(fs.as_ref(), &loaded, name)?;
    let mut out = Vec::new();
    for deck in decks {
        let artifacts = render::prepare_html(&fs, &loaded, &deck)?;
        out.push((deck.name, artifacts));
    }
    Ok(out)
}
