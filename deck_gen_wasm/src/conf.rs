//! Compile-time knobs for the browser editor.
//!
//! URLs, storage keys, import policy, and UI delays used to live next to the
//! code that read them. They are gathered here so a value can be changed in one
//! place without hunting through GitHub, picker, persist, and tooltip modules.
//! Nothing here is mutated at runtime; the WASM build has no process-wide
//! writable statics for these settings.

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

/// Help Markdown: GitHub when missing from the VFS (bundled file as fallback).
pub mod help {
    /// Repo path and VFS path of the help Markdown.
    pub const PATH: &str = "deck_gen_wasm/user-help.md";
}

/// localStorage snapshot of the in-memory workspace.
pub mod session {
    /// Key under which [`crate::persist::Session`] JSON is stored.
    pub const STORAGE_KEY: &str = "deck_gen_wasm.session";
}

/// Folder-import rules for “Load Game”.
pub mod import {
    /// Extensions accepted when copying a disk folder into `games/`.
    pub const ALLOWED_EXTENSIONS: &[&str] = &["md", "json", "json5", "html", "scss"];
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

/// Chrome timing that is not layout CSS.
pub mod ui {
    /// Pointer must stay on an activity-bar control this long before a tooltip.
    pub const TOOLTIP_HOVER_DELAY_MS: u32 = 1500;
}
