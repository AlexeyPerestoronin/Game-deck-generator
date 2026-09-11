//! Native CLI: list / html / pdf. Compiled only with `--features cli`.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use crate::catalog;
use crate::conf::{conf, ChromeSettings, PrintSettings};
use crate::fs::{FileSystem, OsFs};
use crate::render;
use prepare_pdf_host::{CardSize, Chrome, ChromeLocator, Duplex, PdfJob, SheetLayout};

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
    let fs: Arc<dyn FileSystem> = Arc::new(OsFs);
    for (label, artifacts) in crate::prepare_html_named(fs, name)? {
        print_html_logs(&label, &artifacts);
    }
    Ok(())
}

fn pdf_command(name: Option<&str>, duplex_override: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let fs: Arc<dyn FileSystem> = Arc::new(OsFs);
    let loaded = conf()?;
    let chrome = Chrome::launch(&chrome_locator(&loaded.chrome, &loaded.root))?;

    for deck in catalog::find_decks(fs.as_ref(), &loaded, name)? {
        let game = loaded.game_for_deck_name(&deck.name)?;
        let duplex_label = duplex_override.unwrap_or(&game.print.default_duplex);
        let duplex = Duplex::parse(duplex_label).map_err(|err| -> Box<dyn std::error::Error> { err.into() })?;
        fs::create_dir_all(&game.duplex)?;
        let html_artifacts = render::prepare_html(&fs, &loaded, &deck)?;
        print_html_logs(&deck.name, &html_artifacts);
        let job = PdfJob {
            card: CardSize {
                width_mm: deck.card_width_mm(),
                height_mm: deck.card_height_mm(),
            },
            output_dir: deck.output_dir(&loaded)?,
            face_html: html_artifacts.face_html,
            back_html: html_artifacts.back_html,
            face_pdf_name: game.output.face_pdf.clone(),
            back_pdf_name: game.output.back_pdf.clone(),
            duplex_pdf_name: game.output.duplex_pdf.clone(),
            sheet: sheet_from_print(&game.print),
        };
        let pdf = chrome.render_job(&job, duplex)?;
        let collected_pdf = game.duplex.join(format!("{}.pdf", deck.name));
        fs::copy(&pdf.duplex, &collected_pdf)?;
        println!("[{}] {}: {}", deck.name, game.output.face_pdf, pdf.face_pdf.display());
        println!("[{}] {}: {}", deck.name, game.output.back_pdf, pdf.back_pdf.display());
        println!("[{}] {}: {}", deck.name, game.output.duplex_pdf, pdf.duplex.display());
    }
    Ok(())
}

fn print_html_logs(label: &str, artifacts: &render::HtmlArtifacts) {
    println!("[{label}] карточек: {}", artifacts.card_count);
    println!("[{label}] preview: {}", artifacts.preview.display());
    println!("[{label}] face.html: {}", artifacts.face_html.display());
    println!("[{label}] back.html: {}", artifacts.back_html.display());
}

fn sheet_from_print(print: &PrintSettings) -> SheetLayout {
    SheetLayout {
        page_width_mm: print.page_width_mm,
        page_height_mm: print.page_height_mm,
        cols: print.cols,
        rows: print.rows,
        gap_x_mm: print.gap_x_mm,
        gap_y_mm: print.gap_y_mm,
        crop_mark_mm: print.crop_mark_mm,
        crop_mark_gap_mm: print.crop_mark_gap_mm,
        crop_mark_thickness_pt: print.crop_mark_thickness_pt,
    }
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
