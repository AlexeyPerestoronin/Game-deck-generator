//! Compile-time knobs for the browser editor.
//!
//! URLs, storage keys, import policy, file-extension tables, and UI delays used
//! to live next to the code that read them. They are gathered here so a value
//! can be changed in one place without hunting through GitHub, picker, persist,
//! and tooltip modules. Nothing here is mutated at runtime; the WASM build has
//! no process-wide writable statics for these settings.

/// GitHub repository used to list and fetch the `new-game` template.
pub mod github {
    /// `owner/name` on github.com.
    pub const REPO: &str = "AlexeyPerestoronin/Game-deck-generator";
    /// Branch whose git tree and raw files are fetched.
    pub const BRANCH: &str = "master";
}

/// Paths and labels for installing the sample game into `games/`.
pub mod template {
    /// Default folder name under `games/` (suffix `-N` when taken).
    pub const GAME: &str = "new-game";
    /// Git tree prefix of the sample game files.
    pub const PREFIX: &str = "games/new-game/";
    /// Shared games-root config copied only if the workspace has none.
    pub const GAMES_CONF: &str = "games/conf.json5";
    /// Status text when blobs came from GitHub.
    pub const GITHUB_SOURCE_LABEL: &str = "GitHub master";
}

/// Help Markdown compiled into the WASM and copied to the VFS root.
pub mod help {
    /// Workspace-root path of the help Markdown.
    pub const PATH: &str = "user-help.md";
}

/// Browser snapshot of the in-memory workspace.
pub mod session {
    /// Key under which [`crate::persist::Session`] JSON is stored (text tree).
    pub const STORAGE_KEY: &str = "deck_gen_wasm.session";
    /// IndexedDB database for PDF / image bytes (localStorage quota is too small).
    pub const IDB_NAME: &str = "deck_gen_wasm";
    /// Object store inside [`IDB_NAME`].
    pub const IDB_STORE: &str = "binaries";
    /// Single record key: a JS object of path → `Uint8Array`.
    pub const IDB_KEY: &str = "files";
    /// Schema version; bump when the store shape changes.
    pub const IDB_VERSION: u32 = 1;
}

/// Lowercased file-extension tables (no dots). Import, preview, highlight, and
/// icons all read from here so a new type is not added in four match arms.
/// `js` is import-only (via IMPORT_TEXT).
pub mod ext {
    /// Markdown source (preview + highlight). Disk import allows only `md`.
    pub const MARKDOWN: &[&str] = &["md", "markdown"];
    /// HTML source. Disk import allows only `html`.
    pub const HTML: &[&str] = &["html", "htm"];
    /// JSON.
    pub const JSON: &[&str] = &["json"];
    /// JSON5.
    pub const JSON5: &[&str] = &["json5"];
    /// SCSS (import, icon, highlight).
    pub const SCSS: &[&str] = &["scss"];
    /// Extra CSS-family extensions highlighted like SCSS (not imported).
    pub const SASS_CSS: &[&str] = &["sass", "css"];
    /// PDF (preview + icon; not imported from disk).
    pub const PDF: &[&str] = &["pdf"];
    /// Raster / icon images accepted as binary files.
    pub const IMAGE: &[&str] = &["jpg", "jpeg", "png", "ico", "icon"];
    /// Text extensions accepted when copying files from disk into the workspace
    /// (direct pick or inside a folder). Includes `js`, `j2`.
    pub const IMPORT_TEXT: &[&str] = &["md", "json", "json5", "html", "scss", "js", "j2"];
}

/// Folder-import rules for “Load Game” and folder “load file(s)”.
pub mod import {
    /// Text extensions accepted when copying files from disk into the workspace.
    /// Covers both direct file upload and files inside chosen folders.
    /// Image types are listed separately in [`IMAGE_EXTENSIONS`].
    pub const ALLOWED_EXTENSIONS: &[&str] = super::ext::IMPORT_TEXT;
    /// Image extensions accepted alongside [`ALLOWED_EXTENSIONS`].
    /// `jpeg` is the same format as `jpg`; `ico` is the usual name for icon files.
    pub const IMAGE_EXTENSIONS: &[&str] = super::ext::IMAGE;
    /// Images larger than this are refused (100 MiB).
    pub const MAX_IMAGE_BYTES: u64 = 100 * 1024 * 1024;
    /// Fallback folder name when the picker does not supply one.
    pub const DEFAULT_FOLDER_NAME: &str = "game";
    /// How many blocked paths are listed in the reject dialog.
    pub const REJECT_LIST_LIMIT: usize = 8;
}

/// ZIP download of the workspace.
pub mod export {
    /// Default file name offered to the save picker / `<a download>`.
    pub const ZIP_FILENAME: &str = "workspace.zip";
}

/// Bounded async I/O concurrency (WASM has no CPU threads).
pub mod io {
    /// Simultaneous GitHub raw-file fetches when installing `new-game`.
    pub const FETCH_PARALLEL: usize = 8;
    /// Simultaneous `File.text` / `arrayBuffer` reads after import classify.
    pub const FILE_READ_PARALLEL: usize = 8;
}

/// Chrome timing that is not layout CSS.
pub mod ui {
    /// Pointer must stay on an activity-bar control this long before a tooltip.
    pub const TOOLTIP_HOVER_DELAY_MS: u32 = 1500;
    /// Wait this long after the last VFS/selection change before autosave.
    pub const AUTOSAVE_DEBOUNCE_MS: u32 = 300;
    /// After typing pauses, commit the editor draft into VFS (one frame).
    pub const EDIT_FLUSH_MS: u32 = 16;

    /// Annotation texts for the seven activity-bar buttons (used by DelayedTooltip).
    pub const TOOLTIP_CLEAR: &str = "Clear the workspace in this browser.";
    pub const TOOLTIP_DOWNLOAD: &str = "Download the workspace as a ZIP archive.";
    pub const TOOLTIP_LOAD_GAME: &str = "Load a game folder from disk into games/.";
    pub const TOOLTIP_NEW_GAME: &str = "Add a new game from the GitHub master template.";
    pub const TOOLTIP_PREPARE_HTML: &str =
        "Generate HTML preview for every deck in this workspace.";
    pub const TOOLTIP_PREPARE_PDF: &str =
        "Generate card PDFs and A4 duplex sheets for every deck in this workspace.";
    pub const TOOLTIP_SPLIT_PREVIEW: &str =
        "Split for preview. Files stay on the left; previews open on the right.";
    pub const TOOLTIP_FEEDBACK: &str = "Send feedback by email.";

    /// Minimum width (in CSS pixels) of each pane when the editor is split
    /// for preview. Used for the adjustable split resizer.
    pub const SPLIT_PANE_MIN_WIDTH_PX: u32 = 150;

    /// Width (CSS pixels) of the vertical progress ray between the activity bar
    /// and the explorer. Also used as the `.ide` grid column and in the explorer
    /// max-width formula.
    pub const PROGRESS_RAY_WIDTH_PX: u32 = 5;
}

/// Feedback button mailto configuration.
pub mod feedback {
    /// Recipient for user feedback.
    pub const EMAIL: &str = "Alexey.Perestoronin@yandex.ru";
    /// Subject line for the feedback email.
    pub const SUBJECT: &str = "Game-Deck-Generator Feedback";
    /// Email body template (loaded at compile time).
    pub const TEMPLATE: &str = include_str!("../forms/feedback-template.md");
}
