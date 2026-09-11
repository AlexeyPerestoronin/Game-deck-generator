use leptos::prelude::*;

use crate::fs::file_ext;

#[component]
pub fn FileTypeIcon(name: String) -> impl IntoView {
    match file_ext(&name).map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) if ext == "md" => view! { <MdFileIcon /> }.into_any(),
        Some(ext) if ext == "json" => view! { <JsonFileIcon /> }.into_any(),
        Some(ext) if ext == "json5" => view! { <Json5FileIcon /> }.into_any(),
        Some(ext) if ext == "html" => view! { <HtmlFileIcon /> }.into_any(),
        Some(ext) if ext == "scss" => view! { <ScssFileIcon /> }.into_any(),
        Some(ext) if ext == "pdf" => view! { <PdfFileIcon /> }.into_any(),
        _ => view! { <span class="file-icon-slot" aria-hidden="true"></span> }.into_any(),
    }
}

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

#[component]
pub fn MdFileIcon() -> impl IntoView {
    view! {
        <svg class="file-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="#519aba" d="M2 2.5A1.5 1.5 0 0 1 3.5 1h9A1.5 1.5 0 0 1 14 2.5v11A1.5 1.5 0 0 1 12.5 15h-9A1.5 1.5 0 0 1 2 13.5v-11z"/>
            <path fill="#e8e8e8" d="M4.2 11V5h1.2l1.5 3.6L8.4 5H9.6v6H8.5V7.2L7 10.4H6.3L4.8 7.2V11H4.2zm7.1 0-.9-1.3h-.1V11h-1.1V5h1.8c.9 0 1.5.5 1.5 1.4 0 .6-.3 1.1-.8 1.3l1.1 1.7h-1.3l-.9-1.5h-.3V11h-1zm0-4.3c.3 0 .5-.2.5-.5s-.2-.5-.5-.5h-.6v1h.6z"/>
        </svg>
    }
}

#[component]
pub fn JsonFileIcon() -> impl IntoView {
    view! {
        <svg class="file-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="#cbcb41" d="M3 1.5A1.5 1.5 0 0 1 4.5 0h7A1.5 1.5 0 0 1 13 1.5v13A1.5 1.5 0 0 1 11.5 16h-7A1.5 1.5 0 0 1 3 14.5v-13z"/>
            <path fill="#1e1e1e" d="M6.1 8c0-1.1-.5-1.7-1.5-1.7v-.9c1.5 0 2.4 1 2.4 2.6S6.1 10.6 4.6 10.6v-.9c1 0 1.5-.6 1.5-1.7zm3.8 0c0 1.1.5 1.7 1.5 1.7v.9c-1.5 0-2.4-1-2.4-2.6s.9-2.6 2.4-2.6v.9c-1 0-1.5.6-1.5 1.7z"/>
        </svg>
    }
}

#[component]
pub fn Json5FileIcon() -> impl IntoView {
    view! {
        <svg class="file-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="#d19a66" d="M3 1.5A1.5 1.5 0 0 1 4.5 0h7A1.5 1.5 0 0 1 13 1.5v13A1.5 1.5 0 0 1 11.5 16h-7A1.5 1.5 0 0 1 3 14.5v-13z"/>
            <path fill="#1e1e1e" d="M6.1 7.4c0-1-.5-1.6-1.5-1.6v-.9c1.5 0 2.4.9 2.4 2.5S6.1 9.9 4.6 9.9v-.9c1 0 1.5-.6 1.5-1.6zm3.8 0c0 1 .5 1.6 1.5 1.6v.9c-1.5 0-2.4-.9-2.4-2.5s.9-2.5 2.4-2.5v.9c-1 0-1.5.6-1.5 1.6z"/>
            <text x="8" y="14" text-anchor="middle" fill="#1e1e1e" font-size="5" font-family="Segoe UI, sans-serif" font-weight="700">5</text>
        </svg>
    }
}

#[component]
pub fn HtmlFileIcon() -> impl IntoView {
    view! {
        <svg class="file-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="#e37933" d="M2 1h12l-1.1 12.4L8 15l-4.9-1.6L2 1z"/>
            <path fill="#e8e8e8" d="M8 3.2h3.3l-.2 1.6H8.8l.1 1.2h2.1l-.6 5.2L8 12.3l-2.4-1.1-.2-1.6h1.2l.1.8 1.3.5 1.3-.5.2-1.8H5.4l-.4-3.6H8V3.2z"/>
        </svg>
    }
}

#[component]
pub fn ScssFileIcon() -> impl IntoView {
    view! {
        <svg class="file-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="#c6538c" d="M8 1.2c3.7 0 6.2 1.9 6.2 4.3 0 2.1-1.6 3.2-4.1 3.6l-.2 2.2c-.1.8.2 1.1.8 1.1.7 0 1.2-.4 1.6-1.1l1.1.7c-.6 1.2-1.7 2-3 2-1.8 0-2.8-1-2.6-2.6l.3-2.5C6.3 8.7 4.4 7.6 4.4 5.6 4.4 3.1 6.3 1.2 8 1.2zm0 1.6c-1.3 0-2.2 1-2.2 2.4 0 1.2.8 1.9 2.4 2.4 1.5-.3 2.4-.8 2.4-2.1 0-1.5-.9-2.7-2.6-2.7z"/>
        </svg>
    }
}

#[component]
pub fn PdfFileIcon() -> impl IntoView {
    view! {
        <svg class="file-icon" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
            <path fill="#cc3e44" d="M3 1.5A1.5 1.5 0 0 1 4.5 0H9l4 4v10.5A1.5 1.5 0 0 1 11.5 16h-7A1.5 1.5 0 0 1 3 14.5v-13z"/>
            <path fill="#e8e8e8" d="M9 0v4h4L9 0z"/>
            <text x="8" y="12.5" text-anchor="middle" fill="#fff" font-size="4.2" font-family="Segoe UI, sans-serif" font-weight="700">PDF</text>
        </svg>
    }
}
