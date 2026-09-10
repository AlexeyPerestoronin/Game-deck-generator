use leptos::prelude::*;

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
