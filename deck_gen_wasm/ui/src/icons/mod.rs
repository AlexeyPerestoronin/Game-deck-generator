//! Icons for the activity bar (now PNG state images) and explorer rows.
//!
//! Activity icons are 16×16 <img> elements backed by 32×32 PNGs under
//! icons/buttons/*/ (off/on/click, +active for split). Switched by CSS.
//! Explorer file-type icons remain SVG (separate concern).

mod activity;
mod files;

pub use activity::{
    ClearIcon, DownloadIcon, FeedbackIcon, LoadGameIcon, LocaleIcon, NewGameIcon, PrepareHtmlIcon,
    PreparePdfIcon, SplitPreviewIcon, ThemeIcon,
};
pub use files::FileTypeIcon;
