use std::collections::HashSet;

use leptos::prelude::*;

use super::context_menu::{ChosenCommand, ContextMenu, EntryKind, MenuState};
use super::icons::FileTypeIcon;
use crate::fs::{join_path, Vfs};
use crate::workspace::Workspace;

#[component]
pub fn Explorer(workspace: Workspace) -> impl IntoView {
    let menu = RwSignal::new(None::<MenuState>);

    view! {
        <aside class="explorer">
            <div class="explorer-title-row">
                <span class="explorer-title">"Games"</span>
            </div>
            <header class="explorer-header">
                <div class="explorer-actions">
                    <button class="text-btn" title="New File" on:click=move |_| workspace.create_file()>"+ File"</button>
                    <button class="text-btn" title="New Folder" on:click=move |_| workspace.create_folder()>"+ Folder"</button>
                </div>
            </header>
            <div class="tree" role="tree" on:click=move |_| menu.set(None)>
                <TreeRows workspace=workspace menu=menu />
            </div>
            <footer class="status">{move || workspace.status.get()}</footer>
            <Show when=move || menu.get().is_some()>
                {move || menu.get().map(|state| {
                    view! {
                        <ContextMenu
                            state=state
                            on_dismiss=move |_| menu.set(None)
                            on_command=move |chosen: ChosenCommand| {
                                workspace.run_entry_command(chosen.id, &chosen.path);
                            }
                        />
                    }
                })}
            </Show>
        </aside>
    }
}

#[component]
fn TreeRows(workspace: Workspace, menu: RwSignal<Option<MenuState>>) -> impl IntoView {
    view! {
        <For
            each=move || flatten_tree(&workspace.vfs.get(), &workspace.expanded.get())
            key=|row| row.path.clone()
            children=move |row| {
                let path_for_selected = row.path.clone();
                let path_for_click = row.path.clone();
                let path_for_menu = row.path.clone();
                let is_dir = row.is_dir;
                let kind = if is_dir { EntryKind::Folder } else { EntryKind::File };
                let selected = move || workspace.selected.get().as_deref() == Some(path_for_selected.as_str());
                view! {
                    <button
                        class="tree-row"
                        class:selected=selected
                        class:dir=is_dir
                        style=format!("padding-left: {}px", 8 + row.depth * 14)
                        role="treeitem"
                        on:click=move |_| workspace.select(path_for_click.clone(), is_dir)
                        on:contextmenu=move |ev| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            workspace.select(path_for_menu.clone(), is_dir);
                            menu.set(Some(MenuState {
                                path: path_for_menu.clone(),
                                kind,
                                x: f64::from(ev.client_x()),
                                y: f64::from(ev.client_y()),
                            }));
                        }
                    >
                        <span class="chevron">{if is_dir { if row.expanded { "▾" } else { "▸" } } else { " " }}</span>
                        {if is_dir {
                            view! { <span class="file-icon-slot" aria-hidden="true"></span> }.into_any()
                        } else {
                            view! { <FileTypeIcon name=row.name.clone() /> }.into_any()
                        }}
                        <span class="tree-name">{row.name.clone()}</span>
                    </button>
                }
            }
        />
    }
}

#[derive(Clone, PartialEq)]
struct TreeRow {
    path: String,
    name: String,
    depth: usize,
    is_dir: bool,
    expanded: bool,
}

fn flatten_tree(vfs: &Vfs, expanded: &HashSet<String>) -> Vec<TreeRow> {
    let mut rows = Vec::new();
    push_children(vfs, "", 0, expanded, &mut rows);
    rows
}

fn push_children(
    vfs: &Vfs,
    parent: &str,
    depth: usize,
    expanded: &HashSet<String>,
    rows: &mut Vec<TreeRow>,
) {
    for (name, is_dir) in vfs.children(parent) {
        let path = join_path(parent, &name);
        let is_open = is_dir && expanded.contains(&path);
        rows.push(TreeRow {
            path: path.clone(),
            name,
            depth,
            is_dir,
            expanded: is_open,
        });
        if is_open {
            push_children(vfs, &path, depth + 1, expanded, rows);
        }
    }
}
