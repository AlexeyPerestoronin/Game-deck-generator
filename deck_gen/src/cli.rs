//! Native CLI: list / html / pdf / png. Compiled only with `--features cli`.
//!
//! `list` and `html` stay inside this crate. `pdf` launches Chrome through
//! `prepare_pdf_host` (via [`crate::pdf_engine::HostPdfEngine`]) after the same
//! HTML render. `png` also uses Chrome (CDP screenshot) but consumes the per-card
//! HTML files and writes per-card PNGs. The process filesystem is [`crate::fs::OsFs`].

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use crate::catalog;
use crate::conf::ChromeSettings;
use crate::fs::OsFs;
use crate::pdf_engine::HostPdfEngine;
use prepare_pdf_host::{Chrome, ChromeLocator};
use progress_viewer::ProgressHandler;

#[derive(Parser)]
#[command(name = "deck_gen", about = "Generate card decks from JSON5 + templates")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print known deck names
    List {
        #[arg(long)]
        json: bool,
        /// Game id (optional). If omitted, lists decks from all discovered games. All decks of the game are listed when --deck is omitted.
        #[arg(long)]
        game: Option<String>,
        /// Deck name within the game (optional). If omitted, selects all decks of --game.
        #[arg(long)]
        deck: Option<String>,
    },
    /// Render face/back/preview HTML
    Html {
        /// Game id (required).
        #[arg(long)]
        game: String,
        /// Deck name within the game (optional). If omitted, renders all decks of --game.
        #[arg(long)]
        deck: Option<String>,
        /// Use rayon to process multiple decks in parallel (native builds only; ignored for wasm).
        #[arg(long)]
        concurrency: bool,
    },
    /// Render HTML, then card PDFs and an A4 duplex sheet (needs local Chrome)
    Pdf {
        /// Game id (required).
        #[arg(long)]
        game: String,
        /// Deck name within the game (optional). If omitted, renders all decks of --game.
        #[arg(long)]
        deck: Option<String>,
        #[arg(long)]
        duplex: Option<String>,
        /// Use rayon to process multiple decks in parallel (native builds only; ignored for wasm).
        #[arg(long)]
        concurrency: bool,
    },
    /// Render per-card PNGs (face/back) from per-card HTML (needs local Chrome)
    Png {
        /// Game id (required).
        #[arg(long)]
        game: String,
        /// Deck name within the game (optional). If omitted, renders all decks of --game.
        #[arg(long)]
        deck: Option<String>,
        /// Use rayon to process multiple decks in parallel (native builds only; ignored for wasm).
        #[arg(long)]
        concurrency: bool,
    },
}

/// Parse argv and run `list`, `html`, or `pdf`.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::List { json, game, deck } => {
            let q = deck_query(game.as_deref(), deck.as_deref());
            list_command(json, q.as_deref())?
        }
        Command::Html { game, deck, concurrency } => {
            let q = deck_query(Some(&game), deck.as_deref());
            html_command(q.as_deref(), concurrency)?
        }
        Command::Pdf { game, deck, duplex, concurrency } => {
            let q = deck_query(Some(&game), deck.as_deref());
            pdf_command(q.as_deref(), duplex.as_deref(), concurrency)?
        }
        Command::Png { game, deck, concurrency } => {
            let q = deck_query(Some(&game), deck.as_deref());
            png_command(q.as_deref(), concurrency)?
        }
    }
    Ok(())
}

/// Build the catalog query from CLI --game/--deck (game may be absent only for list).
/// Query is game id (selects all its decks), bare deck name, or "game.deckname".
fn deck_query(game: Option<&str>, deck: Option<&str>) -> Option<String> {
    match (game, deck) {
        (Some(g), Some(d)) => Some(format!("{}.{}", g, d)),
        (Some(g), None) => Some(g.to_string()),
        (None, Some(d)) => Some(d.to_string()),
        (None, None) => None,
    }
}

/// Simple progress handler for CLI that prints percentage updates (once per integer).
/// Uses AtomicI32 so it is safe to share across rayon threads when --concurrency is used.
struct CliProgress(std::sync::atomic::AtomicI32);

impl Default for CliProgress {
    fn default() -> Self {
        Self(std::sync::atomic::AtomicI32::new(-1))
    }
}

impl ProgressHandler for CliProgress {
    fn set(&self, pct: f32) {
        use std::sync::atomic::Ordering;
        let v = pct.clamp(0.0, 100.0).floor() as i32;
        if v != self.0.load(Ordering::Relaxed) {
            self.0.store(v, Ordering::Relaxed);
            println!("[deck_gen] progress: {}%", v);
        }
    }
}

fn list_command(json: bool, query: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = OsFs;
    // Explicit load (no global OnceLock cache) per refactoring decision for CLI.
    let loaded = crate::conf::load(&fs)?;
    let names = catalog::matching_names(&fs, &loaded, query)?;
    if json {
        println!("{}", serde_json::to_string(&names)?);
    } else {
        for item in names {
            println!("{item}");
        }
    }
    Ok(())
}

fn html_command(query: Option<&str>, concurrency: bool) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    let p = CliProgress::default();
    for (label, artifacts) in crate::prepare_html_named(fs, query, concurrency, &p)? {
        print_html_logs(&label, artifacts.card_count, &artifacts.preview, &artifacts.face_html, &artifacts.back_html);
    }
    Ok(())
}

fn pdf_command(query: Option<&str>, duplex_override: Option<&str>, concurrency: bool) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    // Explicit load (avoid global cache in CLI).
    let loaded = crate::conf::load(fs.as_ref())?;
    let chrome = Chrome::launch(&chrome_locator(&loaded.chrome, &loaded.root))?;
    let engine = HostPdfEngine::new(chrome);
    let p = CliProgress::default();
    let artifacts = pollster::block_on(crate::prepare_pdf_named(
        fs,
        &engine,
        query,
        duplex_override,
        concurrency,
        &p,
    ))?;
    for (label, pdf) in artifacts {
        print_html_logs(
            &label,
            pdf.card_count,
            &pdf.preview,
            &pdf.face_html,
            &pdf.back_html,
        );
        println!("[{label}] {}: {}", file_name(&pdf.face_pdf), pdf.face_pdf.display());
        println!("[{label}] {}: {}", file_name(&pdf.back_pdf), pdf.back_pdf.display());
        println!("[{label}] {}: {}", file_name(&pdf.duplex), pdf.duplex.display());
    }
    Ok(())
}

fn png_command(query: Option<&str>, concurrency: bool) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    // Explicit load (avoid global cache in CLI).
    let loaded = crate::conf::load(fs.as_ref())?;
    let chrome = Chrome::launch(&chrome_locator(&loaded.chrome, &loaded.root))?;
    let p = CliProgress::default();
    let artifacts = pollster::block_on(crate::prepare_png_named(fs, &chrome, query, concurrency, &p))?;
    for (label, png) in artifacts {
        print_html_logs(
            &label,
            png.card_count,
            &png.preview,
            &png.face_html,
            &png.back_html,
        );
        println!("[{label}] png dir: {}", png.png_dir.display());
        for n in 1..=png.card_count {
            let face = png.png_dir.join(format!("card-{n}-face.png"));
            let back = png.png_dir.join(format!("card-{n}-back.png"));
            println!("[{label}] {}: {}", file_name(&face), face.display());
            println!("[{label}] {}: {}", file_name(&back), back.display());
        }
    }
    Ok(())
}

fn print_html_logs(
    label: &str,
    card_count: usize,
    preview: &std::path::Path,
    face_html: &std::path::Path,
    back_html: &std::path::Path,
) {
    println!("[{label}] карточек: {card_count}");
    println!("[{label}] preview: {}", preview.display());
    println!("[{label}] face.html: {}", face_html.display());
    println!("[{label}] back.html: {}", back_html.display());
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn chrome_locator(settings: &ChromeSettings, conf_root: &std::path::Path) -> ChromeLocator {
    ChromeLocator {
        env_vars: settings.env_vars.clone(),
        executables: settings
            .executables
            .iter()
            .map(|entry| resolve_maybe_relative(conf_root, entry))
            .collect(),
        home_relative: settings
            .home_relative
            .iter()
            .map(PathBuf::from)
            .collect(),
    }
}

fn resolve_maybe_relative(conf_root: &std::path::Path, entry: &str) -> PathBuf {
    let path = PathBuf::from(entry);
    if path.is_absolute() {
        path
    } else {
        conf_root.join(path)
    }
}
