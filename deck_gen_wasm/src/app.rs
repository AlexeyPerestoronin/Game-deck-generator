use leptos::prelude::*;

use crate::persist::{load_session, save_session};
use crate::ui::{ActivityBar, Editor, Explorer};
use crate::workspace::Workspace;

#[component]
pub fn App() -> impl IntoView {
    let workspace = Workspace::from_session(load_session().unwrap_or_default());

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
