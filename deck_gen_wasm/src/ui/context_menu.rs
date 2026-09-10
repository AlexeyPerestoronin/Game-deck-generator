//! Right-click menu. File and folder command lists are independent so they
//! can grow apart without touching shared match arms.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Folder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuCommand {
    pub id: &'static str,
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

pub fn commands_for(kind: EntryKind) -> &'static [MenuCommand] {
    match kind {
        EntryKind::File => FILE_COMMANDS,
        EntryKind::Folder => FOLDER_COMMANDS,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuState {
    pub path: String,
    pub kind: EntryKind,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChosenCommand {
    pub id: &'static str,
    pub path: String,
}

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
