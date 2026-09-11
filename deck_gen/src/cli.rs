//! Native CLI: list / html / pdf. Compiled only with `--features cli`.
//!
//! `list` and `html` stay inside this crate. `pdf` launches Chrome through
//! `prepare_pdf_host` (via [`crate::pdf_engine::HostPdfEngine`]) after the same
//! HTML render. The process filesystem is [`crate::fs::OsFs`].

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
        /// Exact name, game-relative name, or a prefix
        name: Option<String>,
    },
    /// Render face/back/preview HTML
    Html {
        #[arg(long)]
        name: Option<String>,
    },
    /// Render HTML, then card PDFs and an A4 duplex sheet (needs local Chrome)
    Pdf {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        duplex: Option<String>,
    },
}

/// Parse argv and run `list`, `html`, or `pdf`.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::List { json, name } => list_command(json, name.as_deref())?,
        Command::Html { name } => html_command(name.as_deref())?,
        Command::Pdf { name, duplex } => pdf_command(name.as_deref(), duplex.as_deref())?,
    }
    Ok(())
}

fn list_command(json: bool, name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = OsFs;
    let loaded = conf()?;
    let names = catalog::matching_names(&fs, &loaded, name)?;
    if json {
        println!("{}", serde_json::to_string(&names)?);
    } else {
        for item in names {
            println!("{item}");
        }
    }
    Ok(())
}

fn html_command(name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    for (label, artifacts) in crate::prepare_html_named(fs, name)? {
        print_html_logs(&label, artifacts.card_count, &artifacts.preview, &artifacts.face_html, &artifacts.back_html);
    }
    Ok(())
}

fn pdf_command(name: Option<&str>, duplex_override: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs = Arc::new(OsFs);
    let loaded = conf()?;
    let chrome = Chrome::launch(&chrome_locator(&loaded.chrome, &loaded.root))?;
    let engine = HostPdfEngine::new(chrome);
    let artifacts = pollster::block_on(crate::prepare_pdf_named(
        fs,
        &engine,
        name,
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
