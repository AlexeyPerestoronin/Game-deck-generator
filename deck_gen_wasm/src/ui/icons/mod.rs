//! Inline SVG icons for the activity bar and explorer rows.
//!
//! Toolbar glyphs are 16×16 and inherit `currentColor`. Explorer file-type
//! icons map an extension to a glyph; unknown types render an empty slot.

mod activity;
mod files;

pub use activity::{
    ClearIcon, DownloadIcon, LoadGameIcon, NewGameIcon, PrepareHtmlIcon, PreparePdfIcon, SaveIcon,
};
pub use files::FileTypeIcon;
