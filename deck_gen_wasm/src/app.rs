//! Root Leptos view: activity bar, explorer, and editor over one [`Workspace`].
//!
//! Session restore happens once at construction. If
//! [`crate::conf::help::PATH`] is missing from the VFS it is fetched; if no
//! editor tab is open, that file is shown as a preview. An effect writes the
//! snapshot back to localStorage whenever any of the workspace signals change.

use leptos::prelude::*;

use crate::persist::{load_session, save_session};
use crate::ui::{ActivityBar, Editor, Explorer};
use crate::workspace::Workspace;

/// Shell layout: three panes sharing a [`Workspace`].
#[component]
pub fn App() -> impl IntoView {
    let workspace = Workspace::from_session(load_session().unwrap_or_default());
    workspace.ensure_user_help();

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
