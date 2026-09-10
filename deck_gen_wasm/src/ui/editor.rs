use leptos::prelude::*;

use crate::fs::file_name;
use crate::workspace::Workspace;

#[component]
pub fn Editor(workspace: Workspace) -> impl IntoView {
    let open_file = move || workspace.open_file_path();

    view! {
        <main class="editor">
            <header class="editor-tab">
                {move || match open_file() {
                    Some(path) => file_name(&path).to_string(),
                    None => "No file open".into(),
                }}
            </header>
            <Show
                when=move || open_file().is_some()
                fallback=|| view! { <div class="editor-empty">"Select a file to edit, or create one in the explorer."</div> }
            >
                <textarea
                    class="editor-area"
                    prop:value=move || {
                        open_file()
                            .and_then(|path| workspace.vfs.get().read_file(&path).map(str::to_string))
                            .unwrap_or_default()
                    }
                    on:input=move |ev| {
                        if let Some(path) = open_file() {
                            let value = event_target_value(&ev);
                            workspace.vfs.update(|vfs| {
                                let _ = vfs.write_file(&path, value);
                            });
                        }
                    }
                />
            </Show>
        </main>
    }
}
