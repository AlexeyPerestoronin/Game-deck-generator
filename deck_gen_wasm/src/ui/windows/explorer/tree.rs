//! Flattened explorer rows: one button per visible file or folder.
//!
//! Click selects a single row (and toggles folders). Ctrl+click adds or
//! removes a row from the multi-selection without opening files. A selected
//! folder also paints its visible children. Right-click opens the context
//! menu; copy marks are a separate blue highlight.

use std::collections::HashSet;

use leptos::prelude::*;

use crate::fs::{join_path, Vfs};
use crate::ui::icons::FileTypeIcon;
use crate::ui::menus::{EntryKind, MenuState};
use crate::workspace::copy_plan::row_looks_selected;
use crate::workspace::Workspace;

#[component]
pub(super) fn TreeRows(workspace: Workspace, menu: RwSignal<Option<MenuState>>) -> impl IntoView {
    view! {
        <For
            each=move || flatten_tree(&workspace.vfs.get(), &workspace.expanded.get())
            key=|row| row.path.clone()
            children=move |row| {
                let path_for_selected = row.path.clone();
                let path_for_copy = row.path.clone();
                let path_for_click = row.path.clone();
                let path_for_menu = row.path.clone();
                let is_dir = row.is_dir;
                let kind = if is_dir { EntryKind::Folder } else { EntryKind::File };
                let selected = move || {
                    row_looks_selected(&path_for_selected, &workspace.multi_selected.get())
                };
                let copy_planned = move || {
                    workspace.copy_planned.get().contains(&path_for_copy)
                };
                view! {
                    <button
                        class="tree-row"
                        class:selected=selected
                        class:copy-planned=copy_planned
                        class:dir=is_dir
                        style=format!("padding-left: {}px", 8 + row.depth * 14)
                        role="treeitem"
                        on:click=move |ev| {
                            if ev.ctrl_key() {
                                workspace.toggle_select(path_for_click.clone());
                            } else {
                                workspace.select(path_for_click.clone(), is_dir);
                            }
                        }
                        on:contextmenu=move |ev| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            if !workspace.multi_selected.get().contains(&path_for_menu) {
                                workspace.select(path_for_menu.clone(), is_dir);
                            }
                            menu.set(Some(MenuState {
                                path: path_for_menu.clone(),
                                kind,
                                x: f64::from(ev.client_x()),
                                y: f64::from(ev.client_y()),
                                copy_marked: workspace.copy_planned.get().contains(&path_for_menu),
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
