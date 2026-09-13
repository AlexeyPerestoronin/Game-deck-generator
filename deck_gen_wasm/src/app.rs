//! Root Leptos view: activity bar, explorer, and editor over one [`Workspace`].
//!
//! Session restore waits for IndexedDB binaries (PDF / images) so they are
//! present before the tree renders. Text state comes from localStorage. If
//! [`crate::conf::help::PATH`] is missing from the VFS the bundled help file
//! is copied to the workspace root; if no editor tab is open, that file is
//! shown as a preview. An effect writes the text snapshot back to localStorage
//! whenever any of the workspace signals change (debounced), and rewrites
//! IndexedDB only when the binary fingerprint changes.

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::conf;
use crate::persist::{
    binaries_fingerprint, encode_binaries, load_binaries, load_session, save_encoded, save_session,
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
    let last_fp = RwSignal::new(workspace.vfs.with_untracked(binaries_fingerprint));
    let generation = RwSignal::new(0u32);
    Effect::new(move |_| {
        workspace.vfs.with(|_| {});
        workspace.selected.with(|_| {});
        workspace.expanded.with(|_| {});
        let token = generation.get_untracked().wrapping_add(1);
        generation.set(token);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(conf::ui::AUTOSAVE_DEBOUNCE_MS).await;
            if generation.get_untracked() != token {
                return;
            }
            let _ = save_session(&workspace.snapshot());
            let (fp, encoded) = workspace.vfs.with_untracked(|vfs| {
                (binaries_fingerprint(vfs), encode_binaries(vfs))
            });
            if fp != last_fp.get_untracked() {
                last_fp.set(fp);
                spawn_local(async move {
                    let _ = save_encoded(encoded).await;
                });
            }
        });
    });

    view! {
        <div class="ide">
            <ActivityBar workspace=workspace />
            <Explorer workspace=workspace />
            <Editor workspace=workspace />
        </div>
    }
}
