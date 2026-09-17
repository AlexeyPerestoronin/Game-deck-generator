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
use crate::conf::{conf, ChromeSettings};
use crate::fs::OsFs;
use crate::pdf_engine::HostPdfEngine;
use prepare_pdf_host::{Chrome, ChromeLocator};

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
        /// Game id (required). All decks of the game are listed when --deck is omitted.
        #[arg(long)]
        game: String,
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
    },
    /// Render per-card PNGs (face/back) from per-card HTML (needs local Chrome)
    Png {
        /// Game id (required).
        #[arg(long)]
        game: String,
        /// Deck name within the game (optional). If omitted, renders all decks of --game.
        #[arg(long)]
        deck: Option<String>,
    },
}

/// Parse argv and run `list`, `html`, or `pdf`.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::List { json, game, deck } => {
            let q = deck_query(&game, deck.as_deref());
            list_command(json, Some(&q))?
        }
        Command::Html { game, deck } => {
            let q = deck_query(&game, deck.as_deref());
            html_command(Some(&q))?
        }
        Command::Pdf { game, deck, duplex } => {
            let q = deck_query(&game, deck.as_deref());
            pdf_command(Some(&q), duplex.as_deref())?
        }
        Command::Png { game, deck } => {
            let q = deck_query(&game, deck.as_deref());
            png_command(Some(&q))?
        }
    }
    Ok(())
}

/// Build the catalog query from CLI --game/--deck.
/// Query is game id (all decks) or "game.deckname" (specific deck by its declared name).
fn deck_query(game: &str, deck: Option<&str>) -> String {
    match deck {
        Some(d) => format!("{}.{}", game, d),
        None => game.to_string(),
    }
}

fn list_command(json: bool, query: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = OsFs;
    let loaded = conf()?;
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

fn html_command(query: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    for (label, artifacts) in crate::prepare_html_named(fs, query)? {
        print_html_logs(&label, artifacts.card_count, &artifacts.preview, &artifacts.face_html, &artifacts.back_html);
    }
    Ok(())
}

fn pdf_command(query: Option<&str>, duplex_override: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    let loaded = conf()?;
    let chrome = Chrome::launch(&chrome_locator(&loaded.chrome, &loaded.root))?;
    let engine = HostPdfEngine::new(chrome);
    let artifacts = pollster::block_on(crate::prepare_pdf_named(
        fs,
        &engine,
        query,
        duplex_override,
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

fn png_command(query: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    let loaded = conf()?;
    let chrome = Chrome::launch(&chrome_locator(&loaded.chrome, &loaded.root))?;
    let artifacts = pollster::block_on(crate::prepare_png_named(fs, &chrome, query))?;
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
