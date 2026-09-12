//! Root Leptos view: activity bar, explorer, and editor over one [`Workspace`].
//!
//! Session restore happens once at construction. An empty first visit fetches
//! [`crate::conf::help::PATH`] and opens it as a preview tab. An effect writes
//! the snapshot back to localStorage whenever any of the workspace signals
//! change.

use leptos::prelude::*;

use crate::persist::{load_session, save_session};
use crate::ui::{ActivityBar, Editor, Explorer};
use crate::workspace::Workspace;

/// Shell layout: three panes sharing a [`Workspace`].
#[component]
pub fn App() -> impl IntoView {
    let session = load_session();
    let first_visit = session.is_none();
    let workspace = Workspace::from_session(session.unwrap_or_default());
    if first_visit {
        workspace.open_first_visit_help();
    }

    Effect::new(move |_| {
        let _ = save_session(&workspace.snapshot());
    });

    view! {
        <div class="ide">
            <ActivityBar workspace=workspace />
            <Explorer workspace=workspace />
            <Editor workspace=workspace />
        </div>
    }
}
