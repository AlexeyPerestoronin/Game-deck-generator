//! Compile-time knobs for the browser editor.
//!
//! Central place for constants that control GitHub template install,
//! session storage keys, import allow-lists (text + images), I/O parallelism,
//! AI HTTP 503 retry, and UI timing. Values are never mutated at runtime.
//! Both direct file loads and folder loads consult the lists here
//! (via ALLOWED_EXTENSIONS) so *.js, *.j2 (and future types) are enabled
//! in one place.

// Content previously in api.rs (removed per refactoring rules: one lib.rs is enough).

/// GitHub repository used to list and fetch the `new-game` template.
pub mod github {
    /// `owner/name` on github.com.
    pub const REPO: &str = "AlexeyPerestoronin/Game-deck-generator";
    /// Branch whose git tree and raw files are fetched.
    pub const BRANCH: &str = "master";
}

/// GitHub catalog of published games (Globals in the Games sidebar).
pub mod catalog {
    /// `owner/name` on github.com.
    pub const REPO: &str = "AlexeyPerestoronin/Deck-Games";
    /// Branch whose git tree and raw files are fetched.
    pub const BRANCH: &str = "master";
    /// Folder in the catalog repo that contains games.
    pub const GAMES_DIR: &str = "Games";
    /// File whose presence marks a directory as a game root.
    pub const GAME_MARKER: &str = "game.json5";
    /// Relative folder with poster metadata and assets.
    pub const PREVIEW_DIR: &str = "rules/preview";
    /// JSON5 metadata file inside [`PREVIEW_DIR`].
    pub const INFO_FILE: &str = "info.json5";
    /// Bundled HTML shown when a game has no poster file.
    pub const DEFAULT_PREVIEW: &str = "default-preview.html";
    /// Bundled PNG shown when a game has no icon file.
    pub const DEFAULT_ICON: &str = "default-icon.png";
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

/// Help Markdown compiled into the WASM and copied under `help/` in the VFS.
pub mod help {
    /// Workspace folder that holds all help Markdown files.
    pub const DIR: &str = "help";

    /// `help/<stem>-<locale>.md`
    pub(crate) fn file(stem: &str, locale: &str) -> String {
        format!("{DIR}/{stem}-{locale}.md")
    }

    /// VFS path of the user-help file for `locale` (`en` / `ru`).
    pub fn path(locale: &str) -> String {
        file("user-help", locale)
    }
}

/// Game-authoring rules for humans and the AI agent.
pub mod game_help {
    /// VFS path of the game-help file for `locale` (`en` / `ru`).
    pub fn path(locale: &str) -> String {
        super::help::file("game-help", locale)
    }
}

/// AI model configs and run logs in the VFS root.
pub mod ai {
    /// Folder of one-json5-per-model files.
    pub const DIR: &str = "ai-models";
    /// Folder in Deck-Games with model json5 and AI markdown.
    pub const GITHUB_SETTINGS: &str = "Templates/ai-settings";
    /// VFS path of the AI help Markdown for `locale` (`en` / `ru`).
    pub fn help(locale: &str) -> String {
        super::help::file("ai-help", locale)
    }
    /// VFS path of the create-game agent prompt for `locale` (`en` / `ru`).
    pub fn create_game_pt(locale: &str) -> String {
        super::help::file("create-game-pt", locale)
    }
    /// VFS path of the edit-game agent prompt for `locale` (`en` / `ru`).
    pub fn edit_game_pt(locale: &str) -> String {
        super::help::file("edit-game-pt", locale)
    }
    /// Markdown logs of `run_loop`.
    pub const LOG_DIR: &str = "ai-models/log";
    /// Max POSTs of the same JSON when the provider answers HTTP 503.
    pub const HTTP_503_MAX_RETRIES: u32 = 20;
    /// Wait this long after a 503 before repeating the same POST.
    pub const HTTP_503_RETRY_DELAY_MS: u32 = 3000;
}

/// Browser snapshot of the in-memory workspace.
pub mod session {
    /// Key under which [`crate::persist::Session`] JSON is stored (text tree).
    pub const STORAGE_KEY: &str = "deck_gen_wasm.session";
    /// Key for color theme preference (localStorage only; not in Session).
    pub const THEME_KEY: &str = "deck_gen_wasm.theme";
    /// Key for UI language (EN/RU) preference (localStorage only; not in Session).
    pub const LOCALE_KEY: &str = "deck_gen_wasm.locale";
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

#[cfg(test)]
mod tests {
    #[test]
    fn help_paths_use_help_dir_and_locale() {
        assert_eq!(super::help::DIR, "help");
        assert_eq!(super::help::path("en"), "help/user-help-en.md");
        assert_eq!(super::help::path("ru"), "help/user-help-ru.md");
        assert_eq!(super::game_help::path("en"), "help/game-help-en.md");
        assert_eq!(super::game_help::path("ru"), "help/game-help-ru.md");
        assert_eq!(super::ai::help("en"), "help/ai-help-en.md");
        assert_eq!(super::ai::help("ru"), "help/ai-help-ru.md");
        assert_eq!(super::ai::GITHUB_SETTINGS, "Templates/ai-settings");
        assert_eq!(super::ai::create_game_pt("en"), "help/create-game-pt-en.md");
        assert_eq!(super::ai::create_game_pt("ru"), "help/create-game-pt-ru.md");
        assert_eq!(super::ai::edit_game_pt("en"), "help/edit-game-pt-en.md");
        assert_eq!(super::ai::edit_game_pt("ru"), "help/edit-game-pt-ru.md");
    }

    #[test]
    fn ai_http_503_retry_defaults() {
        assert_eq!(super::ai::HTTP_503_MAX_RETRIES, 20);
        assert_eq!(super::ai::HTTP_503_RETRY_DELAY_MS, 3000);
    }
}
