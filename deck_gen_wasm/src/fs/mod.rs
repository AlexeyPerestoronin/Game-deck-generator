//! In-memory workspace tree.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod path;

pub use path::{file_name, join_path, parent_path, split_path};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    File { content: String },
    Dir { children: BTreeMap<String, Node> },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vfs {
    root: BTreeMap<String, Node>,
}

impl Vfs {
    pub fn mkdir(&mut self, path: &str) -> Result<(), String> {
        let parts = split_path(path)?;
        if parts.is_empty() {
            return Err("Cannot create the workspace root".into());
        }
        ensure_dir(&mut self.root, &parts)?;
        Ok(())
    }

    pub fn create_file(&mut self, path: &str) -> Result<(), String> {
        let parts = split_path(path)?;
        let Some((name, parent_parts)) = parts.split_last() else {
            return Err("Cannot create a file at the workspace root".into());
        };
        let parent = ensure_dir(&mut self.root, parent_parts)?;
        match parent.entry(name.to_string()) {
            std::collections::btree_map::Entry::Occupied(_) => {
                Err(format!("'{path}' already exists"))
            }
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(Node::File {
                    content: String::new(),
                });
                Ok(())
            }
        }
    }

    pub fn write_file(&mut self, path: &str, content: String) -> Result<(), String> {
        match node_at_mut(&mut self.root, &split_path(path)?)? {
            Node::File { content: slot } => {
                *slot = content;
                Ok(())
            }
            Node::Dir { .. } => Err(format!("'{path}' is a folder")),
        }
    }

    /// Create or replace a file, creating parent folders as needed.
    pub fn put_file(&mut self, path: &str, content: String) -> Result<(), String> {
        let parts = split_path(path)?;
        let Some((name, parent_parts)) = parts.split_last() else {
            return Err("Cannot write a file at the workspace root".into());
        };
        let parent = ensure_dir(&mut self.root, parent_parts)?;
        if matches!(parent.get(*name), Some(Node::Dir { .. })) {
            return Err(format!("'{path}' is a folder"));
        }
        parent.insert(name.to_string(), Node::File { content });
        Ok(())
    }

    pub fn exists(&self, path: &str) -> bool {
        if path.is_empty() {
            return true;
        }
        let Ok(parts) = split_path(path) else {
            return false;
        };
        node_at(&self.root, &parts).is_ok()
    }

    pub fn read_file(&self, path: &str) -> Option<&str> {
        let parts = split_path(path).ok()?;
        match node_at(&self.root, &parts) {
            Ok(Node::File { content }) => Some(content.as_str()),
            _ => None,
        }
    }

    pub fn is_dir(&self, path: &str) -> bool {
        if path.is_empty() {
            return true;
        }
        let Ok(parts) = split_path(path) else {
            return false;
        };
        matches!(node_at(&self.root, &parts), Ok(Node::Dir { .. }))
    }

    pub fn is_file(&self, path: &str) -> bool {
        self.read_file(path).is_some()
    }

    pub fn children(&self, path: &str) -> Vec<(String, bool)> {
        let Some(map) = dir_children(&self.root, path) else {
            return Vec::new();
        };
        map.iter()
            .map(|(name, node)| (name.clone(), matches!(node, Node::Dir { .. })))
            .collect()
    }

    pub fn files_and_dirs(&self) -> (Vec<String>, Vec<(String, String)>) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        collect_entries("", &self.root, &mut dirs, &mut files);
        (dirs, files)
    }
}

fn dir_children<'a>(
    root: &'a BTreeMap<String, Node>,
    path: &str,
) -> Option<&'a BTreeMap<String, Node>> {
    if path.is_empty() {
        return Some(root);
    }
    let parts = split_path(path).ok()?;
    match node_at(root, &parts) {
        Ok(Node::Dir { children }) => Some(children),
        _ => None,
    }
}

fn ensure_dir<'a>(
    children: &'a mut BTreeMap<String, Node>,
    parts: &[&str],
) -> Result<&'a mut BTreeMap<String, Node>, String> {
    if parts.is_empty() {
        return Ok(children);
    }
    let name = parts[0];
    let rest = &parts[1..];
    let node = children.entry(name.to_string()).or_insert_with(|| Node::Dir {
        children: BTreeMap::new(),
    });
    match node {
        Node::Dir { children } => ensure_dir(children, rest),
        Node::File { .. } => Err(format!("'{name}' is a file")),
    }
}

fn node_at<'a>(children: &'a BTreeMap<String, Node>, parts: &[&str]) -> Result<&'a Node, String> {
    let Some((first, rest)) = parts.split_first() else {
        return Err("empty path".into());
    };
    let node = children
        .get(*first)
        .ok_or_else(|| format!("'{first}' not found"))?;
    if rest.is_empty() {
        return Ok(node);
    }
    match node {
        Node::Dir { children } => node_at(children, rest),
        Node::File { .. } => Err(format!("'{first}' is a file")),
    }
}

fn node_at_mut<'a>(
    children: &'a mut BTreeMap<String, Node>,
    parts: &[&str],
) -> Result<&'a mut Node, String> {
    let Some((first, rest)) = parts.split_first() else {
        return Err("empty path".into());
    };
    let node = children
        .get_mut(*first)
        .ok_or_else(|| format!("'{first}' not found"))?;
    if rest.is_empty() {
        return Ok(node);
    }
    match node {
        Node::Dir { children } => node_at_mut(children, rest),
        Node::File { .. } => Err(format!("'{first}' is a file")),
    }
}

fn collect_entries(
    prefix: &str,
    children: &BTreeMap<String, Node>,
    dirs: &mut Vec<String>,
    files: &mut Vec<(String, String)>,
) {
    for (name, node) in children {
        let path = join_path(prefix, name);
        match node {
            Node::Dir { children } => {
                dirs.push(path.clone());
                collect_entries(&path, children, dirs, files);
            }
            Node::File { content } => files.push((path, content.clone())),
        }
    }
}
