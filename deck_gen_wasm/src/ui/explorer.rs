use std::collections::HashSet;

use leptos::prelude::*;

use crate::fs::{join_path, Vfs};
use crate::workspace::Workspace;

#[component]
pub fn Explorer(workspace: Workspace) -> impl IntoView {
    view! {
        <aside class="explorer">
            <header class="explorer-header">
                <span class="explorer-title">"EXPLORER"</span>
                <div class="explorer-actions">
                    <button class="text-btn" title="New File" on:click=move |_| workspace.create_file()>"+ File"</button>
                    <button class="text-btn" title="New Folder" on:click=move |_| workspace.create_folder()>"+ Folder"</button>
                </div>
            </header>
            <div class="tree" role="tree">
                <TreeRows workspace=workspace />
            </div>
            <footer class="status">{move || workspace.status.get()}</footer>
        </aside>
    }
}

#[component]
fn TreeRows(workspace: Workspace) -> impl IntoView {
    view! {
        <For
            each=move || flatten_tree(&workspace.vfs.get(), &workspace.expanded.get())
            key=|row| row.path.clone()
            children=move |row| {
                let path_for_selected = row.path.clone();
                let path_for_click = row.path.clone();
                let is_dir = row.is_dir;
                let selected = move || workspace.selected.get().as_deref() == Some(path_for_selected.as_str());
                view! {
                    <button
                        class="tree-row"
                        class:selected=selected
                        class:dir=is_dir
                        style=format!("padding-left: {}px", 8 + row.depth * 14)
                        role="treeitem"
                        on:click=move |_| workspace.select(path_for_click.clone(), is_dir)
                    >
                        <span class="chevron">{if is_dir { if row.expanded { "▾" } else { "▸" } } else { " " }}</span>
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
