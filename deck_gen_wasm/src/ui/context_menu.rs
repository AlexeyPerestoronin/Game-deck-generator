//! Right-click menu for explorer entries.
//!
//! File and folder command lists are independent tables so they can grow
//! apart without shared match arms. The menu is positioned at the click and
//! dismissed by backdrop click or choosing a command.

use leptos::prelude::*;

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

/// Command table for `kind`.
pub fn commands_for(kind: EntryKind) -> &'static [MenuCommand] {
    match kind {
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
    let commands = commands_for(state.kind);
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
