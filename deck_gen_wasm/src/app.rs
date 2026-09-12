//! Root Leptos view: activity bar, explorer, and editor over one [`Workspace`].
//!
//! Session restore waits for IndexedDB binaries (PDF / images) so they are
//! present before the tree renders. Text state comes from localStorage. If
//! [`crate::conf::help::PATH`] is missing from the VFS the bundled help file
//! is copied to the workspace root; if no editor tab is open, that file is
//! shown as a preview. An effect writes the text snapshot back to localStorage
//! whenever any of the workspace signals change, and rewrites IndexedDB only
//! when the binary fingerprint changes.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::persist::{
    binaries_fingerprint, load_binaries, load_session, save_binaries, save_session,
};
use crate::ui::{ActivityBar, Editor, Explorer};
use crate::workspace::Workspace;

/// Shell layout: load persisted state, then the three panes.
#[component]
pub fn App() -> impl IntoView {
    let workspace_slot = RwSignal::new(None::<Workspace>);
    spawn_local(async move {
        let mut session = load_session().unwrap_or_default();
        if let Ok(entries) = load_binaries().await {
            session.vfs.restore_binaries(entries);
        }
        let workspace = Workspace::from_session(session);
        workspace.ensure_user_help();
        workspace_slot.set(Some(workspace));
    });

    view! {
        {move || match workspace_slot.get() {
            None => view! {
                <div class="ide">
                    <div class="editor-empty">"Loading workspace…"</div>
                </div>
            }
            .into_any(),
            Some(workspace) => view! { <LoadedApp workspace=workspace /> }.into_any(),
        }}
    }
}

/// Autosave + three panes once localStorage and IndexedDB have been merged.
#[component]
fn LoadedApp(workspace: Workspace) -> impl IntoView {
    let last_fp = RwSignal::new(binaries_fingerprint(
        &workspace.vfs.get_untracked().binary_entries(),
    ));
    Effect::new(move |_| {
        let _ = save_session(&workspace.snapshot());
        let entries = workspace.vfs.get().binary_entries();
        let fp = binaries_fingerprint(&entries);
        if fp != last_fp.get_untracked() {
            last_fp.set(fp);
            spawn_local(async move {
                let _ = save_binaries(&entries).await;
            });
        }
    });

    view! {
        <div class="ide">
            <ActivityBar workspace=workspace />
            <Explorer workspace=workspace />
            <Editor workspace=workspace />
        </div>
    }
}
