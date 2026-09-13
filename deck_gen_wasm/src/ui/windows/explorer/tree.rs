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
            each=move || {
                workspace.vfs.with(|vfs| {
                    workspace.expanded.with(|expanded| flatten_tree(vfs, expanded))
                })
            }
            key=|row| row.path.clone()
            children=move |row| {
                let path = row.path.clone();
                let is_dir = row.is_dir;
                let kind = if is_dir { EntryKind::Folder } else { EntryKind::File };
                let selected_path = path.clone();
                let copy_path = path.clone();
                let click_path = path.clone();
                let menu_path = path.clone();
                let selected = move || {
                    row_looks_selected(&selected_path, &workspace.multi_selected.get())
                };
                let copy_planned = move || {
                    workspace.copy_planned.get().contains(&copy_path)
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
                            let path = click_path.to_string();
                            if ev.ctrl_key() {
                                workspace.toggle_select(path);
                            } else {
                                workspace.select(path, is_dir);
                            }
                        }
                        on:contextmenu=move |ev| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            if !workspace.multi_selected.get().contains(&menu_path) {
                                workspace.select(menu_path.clone(), is_dir);
                            }
                            menu.set(Some(MenuState {
                                path: menu_path.clone(),
                                kind,
                                x: f64::from(ev.client_x()),
                                y: f64::from(ev.client_y()),
                                copy_marked: workspace.copy_planned.get().contains(&menu_path),
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
        let path = join_path(parent, name);
        let is_open = is_dir && expanded.contains(&path);
        rows.push(TreeRow {
            path: path.clone(),
            name: name.to_string(),
            depth,
            is_dir,
            expanded: is_open,
        });
        if is_open {
            push_children(vfs, &path, depth + 1, expanded, rows);
        }
    }
}
