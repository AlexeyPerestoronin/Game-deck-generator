//! In-memory workspace tree.
//!
//! [`Vfs`] is a `BTreeMap` of [`Node`] (file or directory). Paths are the
//! `/`-separated strings from [`path`]. Mutating methods return `Result<_, String>`
//! so the UI can show the message as status. [`VfsFs`] adapts this tree to
//! [`deck_gen::FileSystem`] for HTML generation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod path;
mod vfs_fs;

pub use path::{
    file_ext, file_name, join_path, parent_path, rewrite_prefix, split_path, unique_name,
};
pub use vfs_fs::VfsFs;

/// A file body or a sorted map of children.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    /// UTF-8 file contents.
    File { content: String },
    /// Raw bytes (PDF and images). Kept in IndexedDB, not localStorage.
    Binary { data: Vec<u8> },
    /// Directory keyed by a single path segment.
    Dir { children: BTreeMap<String, Node> },
}

/// Workspace root: the map of top-level names (usually `games`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vfs {
    root: BTreeMap<String, Node>,
}

impl Vfs {
    /// Create `path` and any missing parent directories.
    pub fn mkdir(&mut self, path: &str) -> Result<(), String> {
        let parts = split_path(path)?;
        if parts.is_empty() {
            return Err("Cannot create the workspace root".into());
        }
        ensure_dir(&mut self.root, &parts)?;
        Ok(())
    }

    /// Create an empty file; error if `path` already exists.
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

    /// Replace the body of an existing UTF-8 file.
    pub fn write_file(&mut self, path: &str, content: String) -> Result<(), String> {
        match node_at_mut(&mut self.root, &split_path(path)?)? {
            Node::File { content: slot } => {
                *slot = content;
                Ok(())
            }
            Node::Binary { .. } => Err(format!("'{path}' is a binary file")),
            Node::Dir { .. } => Err(format!("'{path}' is a folder")),
        }
    }

    /// Create or replace a UTF-8 file, creating parent folders as needed.
    pub fn put_file(&mut self, path: &str, content: String) -> Result<(), String> {
        self.put_node(path, Node::File { content })
    }

    /// Create or replace a binary file, creating parent folders as needed.
    pub fn put_bytes(&mut self, path: &str, data: Vec<u8>) -> Result<(), String> {
        self.put_node(path, Node::Binary { data })
    }

    fn put_node(&mut self, path: &str, node: Node) -> Result<(), String> {
        let parts = split_path(path)?;
        let Some((name, parent_parts)) = parts.split_last() else {
            return Err("Cannot write a file at the workspace root".into());
        };
        let parent = ensure_dir(&mut self.root, parent_parts)?;
        if matches!(parent.get(*name), Some(Node::Dir { .. })) {
            return Err(format!("'{path}' is a folder"));
        }
        parent.insert(name.to_string(), node);
        Ok(())
    }

    /// Whether `path` is the root or an existing node.
    pub fn exists(&self, path: &str) -> bool {
        if path.is_empty() {
            return true;
        }
        let Ok(parts) = split_path(path) else {
            return false;
        };
        node_at(&self.root, &parts).is_ok()
    }

    /// UTF-8 file body, or `None` if missing, binary, or a directory.
    pub fn read_file(&self, path: &str) -> Option<&str> {
        let parts = split_path(path).ok()?;
        match node_at(&self.root, &parts) {
            Ok(Node::File { content }) => Some(content.as_str()),
            _ => None,
        }
    }

    /// File bytes (text as UTF-8, or binary), or `None` if missing/directory.
    pub fn read_bytes(&self, path: &str) -> Option<&[u8]> {
        let parts = split_path(path).ok()?;
        match node_at(&self.root, &parts) {
            Ok(Node::File { content }) => Some(content.as_bytes()),
            Ok(Node::Binary { data }) => Some(data.as_slice()),
            _ => None,
        }
    }

    /// Whether `path` is a binary file.
    pub fn is_binary(&self, path: &str) -> bool {
        let Ok(parts) = split_path(path) else {
            return false;
        };
        matches!(node_at(&self.root, &parts), Ok(Node::Binary { .. }))
    }

    /// Snapshot without binary files (localStorage must not hold PDFs / images).
    pub fn without_binaries(&self) -> Self {
        Self {
            root: strip_binaries(&self.root),
        }
    }

    /// Paths and bytes of every [`Node::Binary`] file, in tree order.
    pub fn binary_entries(&self) -> Vec<(String, Vec<u8>)> {
        let mut files = Vec::new();
        collect_binaries("", &self.root, &mut files);
        files
    }

    /// Write binary files back onto a tree that was stripped for localStorage.
    pub fn restore_binaries(&mut self, entries: impl IntoIterator<Item = (String, Vec<u8>)>) {
        for (path, data) in entries {
            let _ = self.put_bytes(&path, data);
        }
    }

    /// Whether `path` is the root or an existing directory.
    pub fn is_dir(&self, path: &str) -> bool {
        if path.is_empty() {
            return true;
        }
        let Ok(parts) = split_path(path) else {
            return false;
        };
        matches!(node_at(&self.root, &parts), Ok(Node::Dir { .. }))
    }

    /// Whether `path` is an existing file.
    pub fn is_file(&self, path: &str) -> bool {
        let Ok(parts) = split_path(path) else {
            return false;
        };
        matches!(
            node_at(&self.root, &parts),
            Ok(Node::File { .. } | Node::Binary { .. })
        )
    }

    /// Immediate children as `(name, is_dir)`, sorted by [`BTreeMap`] order.
    pub fn children(&self, path: &str) -> Vec<(String, bool)> {
        let Some(map) = dir_children(&self.root, path) else {
            return Vec::new();
        };
        map.iter()
            .map(|(name, node)| (name.clone(), matches!(node, Node::Dir { .. })))
            .collect()
    }

    /// Depth-first listing: directory paths, then `(path, bytes)` files.
    pub fn files_and_dirs(&self) -> (Vec<String>, Vec<(String, Vec<u8>)>) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        collect_entries("", &self.root, &mut dirs, &mut files);
        (dirs, files)
    }

    /// Delete a file or directory (and its descendants).
    pub fn remove(&mut self, path: &str) -> Result<(), String> {
        let parts = split_path(path)?;
        let Some((name, parent_parts)) = parts.split_last() else {
            return Err("Cannot delete the workspace root".into());
        };
        let parent = parent_map_mut(&mut self.root, parent_parts)?;
        parent
            .remove(*name)
            .ok_or_else(|| format!("'{path}' not found"))?;
        Ok(())
    }

    /// Rename one segment of `path`; returns the new full path.
    pub fn rename(&mut self, path: &str, new_name: &str) -> Result<String, String> {
        let new_name = new_name.trim();
        if !is_single_segment_name(new_name) {
            return Err("Name must be a single path segment".into());
        }
        let parts = split_path(path)?;
        let Some((old_name, parent_parts)) = parts.split_last() else {
            return Err("Cannot rename the workspace root".into());
        };
        if *old_name == new_name {
            return Ok(path.to_string());
        }
        let parent = parent_map_mut(&mut self.root, parent_parts)?;
        if parent.contains_key(new_name) {
            return Err(format!("'{new_name}' already exists"));
        }
        let node = parent
            .remove(*old_name)
            .ok_or_else(|| format!("'{path}' not found"))?;
        parent.insert(new_name.to_string(), node);
        Ok(join_path(&parent_path(path), new_name))
    }
}

fn is_single_segment_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\\')
}

fn parent_map_mut<'a>(
    root: &'a mut BTreeMap<String, Node>,
    parent_parts: &[&str],
) -> Result<&'a mut BTreeMap<String, Node>, String> {
    if parent_parts.is_empty() {
        return Ok(root);
    }
    match node_at_mut(root, parent_parts)? {
        Node::Dir { children } => Ok(children),
        Node::File { .. } | Node::Binary { .. } => Err("parent is a file".into()),
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
    let node = children
        .entry(name.to_string())
        .or_insert_with(|| Node::Dir {
            children: BTreeMap::new(),
        });
    match node {
        Node::Dir { children } => ensure_dir(children, rest),
        Node::File { .. } | Node::Binary { .. } => Err(format!("'{name}' is a file")),
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
        Node::File { .. } | Node::Binary { .. } => Err(format!("'{first}' is a file")),
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
        Node::File { .. } | Node::Binary { .. } => Err(format!("'{first}' is a file")),
    }
}

#[cfg(test)]
mod vfs_tests {
    use super::*;

    #[test]
    fn remove_file_and_rename_folder() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/data.json5", "x".into()).unwrap();
        vfs.rename("games/a", "b").unwrap();
        assert!(vfs.is_file("games/b/data.json5"));
        assert!(!vfs.exists("games/a"));
        vfs.remove("games/b").unwrap();
        assert!(!vfs.exists("games/b"));
    }

    #[test]
    fn binary_files_round_trip_and_strip() {
        let mut vfs = Vfs::default();
        vfs.put_bytes("games/a/face.pdf", vec![0x25, 0x50, 0x44, 0x46])
            .unwrap();
        assert!(vfs.is_file("games/a/face.pdf"));
        assert!(vfs.is_binary("games/a/face.pdf"));
        assert!(vfs.read_file("games/a/face.pdf").is_none());
        assert_eq!(
            vfs.read_bytes("games/a/face.pdf"),
            Some(&[0x25, 0x50, 0x44, 0x46][..])
        );
        let stripped = vfs.without_binaries();
        assert!(!stripped.exists("games/a/face.pdf"));
        assert!(stripped.is_dir("games/a"));
    }

    #[test]
    fn binaries_restore_onto_stripped_tree() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/data.json5", "x".into()).unwrap();
        vfs.put_bytes("games/a/face.pdf", vec![0x25, 0x50]).unwrap();
        vfs.put_bytes("games/a/logo.png", vec![0x89, 0x50]).unwrap();
        let entries = vfs.binary_entries();
        assert_eq!(entries.len(), 2);
        let mut stripped = vfs.without_binaries();
        assert!(!stripped.exists("games/a/face.pdf"));
        stripped.restore_binaries(entries);
        assert_eq!(
            stripped.read_bytes("games/a/face.pdf"),
            Some(&[0x25, 0x50][..])
        );
        assert_eq!(
            stripped.read_bytes("games/a/logo.png"),
            Some(&[0x89, 0x50][..])
        );
        assert_eq!(stripped.read_file("games/a/data.json5"), Some("x"));
    }
}

fn collect_entries(
    prefix: &str,
    children: &BTreeMap<String, Node>,
    dirs: &mut Vec<String>,
    files: &mut Vec<(String, Vec<u8>)>,
) {
    for (name, node) in children {
        let path = join_path(prefix, name);
        match node {
            Node::Dir { children } => {
                dirs.push(path.clone());
                collect_entries(&path, children, dirs, files);
            }
            Node::File { content } => files.push((path, content.as_bytes().to_vec())),
            Node::Binary { data } => files.push((path, data.clone())),
        }
    }
}

fn collect_binaries(
    prefix: &str,
    children: &BTreeMap<String, Node>,
    files: &mut Vec<(String, Vec<u8>)>,
) {
    for (name, node) in children {
        let path = join_path(prefix, name);
        match node {
            Node::Dir { children } => collect_binaries(&path, children, files),
            Node::Binary { data } => files.push((path, data.clone())),
            Node::File { .. } => {}
        }
    }
}

fn strip_binaries(children: &BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    children
        .iter()
        .filter_map(|(name, node)| match node {
            Node::Binary { .. } => None,
            Node::File { .. } => Some((name.clone(), node.clone())),
            Node::Dir { children } => Some((
                name.clone(),
                Node::Dir {
                    children: strip_binaries(children),
                },
            )),
        })
        .collect()
}
