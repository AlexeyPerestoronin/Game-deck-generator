//! 16×16 glyphs for activity-bar buttons. Paths inherit `currentColor`.

use leptos::prelude::*;

/// Activity-bar save (localStorage) glyph.
#[component]
pub fn SaveIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M13.5 2h-11A1.5 1.5 0 0 0 1 3.5v9A1.5 1.5 0 0 0 2.5 14h11a1.5 1.5 0 0 0 1.5-1.5v-9A1.5 1.5 0 0 0 13.5 2zM3 4h7v3H3V4zm10 8.5a.5.5 0 0 1-.5.5h-9a.5.5 0 0 1-.5-.5V8h10v4.5z"
            />
        </svg>
    }
}

/// Activity-bar ZIP download glyph.
#[component]
pub fn DownloadIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M8 2a.75.75 0 0 1 .75.75v6.19l2.22-2.22a.75.75 0 1 1 1.06 1.06l-3.5 3.5a.75.75 0 0 1-1.06 0l-3.5-3.5a.75.75 0 0 1 1.06-1.06L7.25 8.94V2.75A.75.75 0 0 1 8 2zM3 12.5A.5.5 0 0 1 3.5 12h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5z"
            />
        </svg>
    }
}

/// Activity-bar “load folder from disk” glyph.
#[component]
pub fn LoadGameIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M1.5 3.5A1.5 1.5 0 0 1 3 2h3.2c.3 0 .58.16.74.42L7.7 3.5H13A1.5 1.5 0 0 1 14.5 5v7A1.5 1.5 0 0 1 13 13.5H3A1.5 1.5 0 0 1 1.5 12V3.5z"
            />
            <path fill="#1e1e1e" d="M8 6.2v3.2h1.6L8 11.2 6.4 9.4H8V6.2z"/>
        </svg>
    }
}

/// Activity-bar “new game from template” glyph.
#[component]
pub fn NewGameIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M2.5 2A1.5 1.5 0 0 0 1 3.5v9A1.5 1.5 0 0 0 2.5 14H8v-1H2.5a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5H9v3.5A.5.5 0 0 0 9.5 7H13v1h1V6.5a.5.5 0 0 0-.15-.35l-3-3A.5.5 0 0 0 10.5 3H2.5zM10 4.21 12.79 7H10V4.21zM12 10v2h2v1h-2v2h-1v-2h-2v-1h2v-2h1z"
            />
        </svg>
    }
}

/// Activity-bar `prepare_pdf` glyph.
#[component]
pub fn PreparePdfIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M3 1.5A1.5 1.5 0 0 1 4.5 0H9l4 4v10.5A1.5 1.5 0 0 1 11.5 16h-7A1.5 1.5 0 0 1 3 14.5v-13z"
            />
            <path fill="#1e1e1e" d="M9 0v4h4L9 0z"/>
            <text x="8" y="12.2" text-anchor="middle" fill="#1e1e1e" font-size="4.4" font-family="Segoe UI, sans-serif" font-weight="700">PDF</text>
        </svg>
    }
}

/// Activity-bar `prepare_html` glyph.
#[component]
pub fn PrepareHtmlIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M3 1.5A1.5 1.5 0 0 1 4.5 0H9l4 4v10.5A1.5 1.5 0 0 1 11.5 16h-7A1.5 1.5 0 0 1 3 14.5v-13z"
            />
            <path fill="#1e1e1e" d="M9 0v4h4L9 0z"/>
            <path fill="#1e1e1e" d="M6.2 8.2 8 10l1.8-1.8.8.8L8 11.8 5.4 9l.8-.8z"/>
        </svg>
    }
}

/// Activity-bar clear-workspace glyph.
#[component]
pub fn ClearIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path
                fill="currentColor"
                d="M6.5 1.75A.75.75 0 0 1 7.25 1h1.5a.75.75 0 0 1 .75.75V3h3.75a.75.75 0 0 1 0 1.5h-.34l-.7 8.38A1.75 1.75 0 0 1 10.47 14.5H5.53a1.75 1.75 0 0 1-1.74-1.62l-.7-8.38h-.34a.75.75 0 0 1 0-1.5H6.5V1.75zm1 1.5V3h1V3.25h-1zM4.62 4.5l.68 8.2a.25.25 0 0 0 .25.22h4.9a.25.25 0 0 0 .25-.22l.68-8.2H4.62z"
            />
        </svg>
    }
}
