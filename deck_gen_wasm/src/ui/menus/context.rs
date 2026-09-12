//! Right-click menu for explorer entries.
//!
//! File and folder command lists are independent tables so they can grow
//! apart without shared match arms. The menu is positioned at the click and
//! dismissed by backdrop click or choosing a command.

use leptos::prelude::*;

use crate::fs::file_ext;

/// Whether the context menu was opened on a file or a folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Folder,
}

/// One menu row: stable `id` (handled by the workspace) and visible label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuCommand {
    /// Command id (`rename`, `delete`, …).
    pub id: &'static str,
    /// Text shown in the menu.
    pub label: &'static str,
}

const FILE_COMMANDS: &[MenuCommand] = &[
    MenuCommand {
        id: "rename",
        label: "Rename",
    },
    MenuCommand {
        id: "delete",
        label: "Delete",
    },
];

const FILE_PREVIEW_COMMANDS: &[MenuCommand] = &[
    MenuCommand {
        id: "preview",
        label: "Preview",
    },
    MenuCommand {
        id: "rename",
        label: "Rename",
    },
    MenuCommand {
        id: "delete",
        label: "Delete",
    },
];

const FOLDER_COMMANDS: &[MenuCommand] = &[
    MenuCommand {
        id: "rename",
        label: "Rename",
    },
    MenuCommand {
        id: "delete",
        label: "Delete",
    },
];

fn is_previewable(path: &str) -> bool {
    matches!(
        file_ext(path).map(str::to_ascii_lowercase).as_deref(),
        Some("html" | "htm" | "md" | "markdown" | "pdf")
    )
}

/// Command table for `kind` (files with html/md/pdf also get Preview).
pub fn commands_for(kind: EntryKind, path: &str) -> &'static [MenuCommand] {
    match kind {
        EntryKind::File if is_previewable(path) => FILE_PREVIEW_COMMANDS,
        EntryKind::File => FILE_COMMANDS,
        EntryKind::Folder => FOLDER_COMMANDS,
    }
}

/// Open menu: target path, entry kind, and viewport coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuState {
    /// Path the menu was opened on.
    pub path: String,
    /// File vs folder command list.
    pub kind: EntryKind,
    /// Viewport X of the click.
    pub x: f64,
    /// Viewport Y of the click.
    pub y: f64,
}

/// Command the user picked, plus the path it applies to.
#[derive(Clone, Debug, PartialEq)]
pub struct ChosenCommand {
    /// Command id from [`MenuCommand::id`].
    pub id: &'static str,
    /// Path from [`MenuState::path`].
    pub path: String,
}

/// Floating menu at `state`’s coordinates.
#[component]
pub fn ContextMenu(
    state: MenuState,
    #[prop(into)] on_dismiss: Callback<()>,
    #[prop(into)] on_command: Callback<ChosenCommand>,
) -> impl IntoView {
    let commands = commands_for(state.kind, &state.path);
    let path = state.path.clone();
    view! {
        <div
            class="context-backdrop"
            role="presentation"
            on:click=move |_| on_dismiss.run(())
            on:contextmenu=move |ev| {
                ev.prevent_default();
                on_dismiss.run(());
            }
        >
            <div
                class="context-menu"
                role="menu"
                style=format!("left:{}px;top:{}px", state.x, state.y)
                on:click=move |ev| ev.stop_propagation()
            >
                {commands
                    .iter()
                    .copied()
                    .map(|command| {
                        let path = path.clone();
                        view! {
                            <button
                                class="context-item"
                                role="menuitem"
                                on:click=move |_| {
                                    on_command.run(ChosenCommand {
                                        id: command.id,
                                        path: path.clone(),
                                    });
                                    on_dismiss.run(());
                                }
                            >
                                {command.label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_menu_for_html_md_pdf() {
        assert_eq!(
            commands_for(EntryKind::File, "games/a/preview.html")[0].id,
            "preview"
        );
        assert_eq!(commands_for(EntryKind::File, "help.MD")[0].id, "preview");
        assert_eq!(
            commands_for(EntryKind::File, "games/a/face.pdf")[0].id,
            "preview"
        );
        assert_eq!(
            commands_for(EntryKind::File, "games/a/data.json5")[0].id,
            "rename"
        );
        assert_eq!(commands_for(EntryKind::Folder, "games/a")[0].id, "rename");
    }
}
