//! In-memory workspace tree.
//!
//! [`Vfs`] is a thin wrapper around a `BTreeMap` of [`Node`] (file or directory).
//! Paths are `/`-separated strings from [`path`]. Mutating methods return
//! `Result<_, String>` so the UI can show the message as status.
//!
//! `Node` is the primary public element of the filesystem. `Vfs` provides the
//! high-level operations (mkdir, read/write, copy, etc.). Binary vs text
//! handling policy lives in callers (see `kind`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::path::{file_name, join_path, parent_path, split_path, unique_name};

/// A file body (arbitrary bytes) or a sorted map of children.
///
/// All files are stored uniformly as bytes. Distinguishing text vs. binary
/// (for preview, persistence strategy, etc.) is the responsibility of the caller,
/// typically using `crate::kind`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    /// File contents as bytes (text is valid UTF-8; images/PDFs are not).
    File { data: Vec<u8> },
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
                slot.insert(Node::File { data: Vec::new() });
                Ok(())
            }
        }
    }

    /// Replace the body of an existing file (stores UTF-8 bytes).
    pub fn write_file(&mut self, path: &str, content: String) -> Result<(), String> {
        match node_at_mut(&mut self.root, &split_path(path)?)? {
            Node::File { data: slot } => {
                *slot = content.into_bytes();
                Ok(())
            }
            Node::Dir { .. } => Err(format!("'{path}' is a folder")),
        }
    }

    /// Create or replace a file (UTF-8), creating parent folders as needed.
    pub fn put_file(&mut self, path: &str, content: String) -> Result<(), String> {
        self.put_node(path, Node::File { data: content.into_bytes() })
    }

    /// Create or replace a file with arbitrary bytes, creating parent folders as needed.
    pub fn put_bytes(&mut self, path: &str, data: Vec<u8>) -> Result<(), String> {
        self.put_node(path, Node::File { data })
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

    /// File body as UTF-8 string, or `None` if missing, not valid UTF-8, or a directory.
    pub fn read_file(&self, path: &str) -> Option<&str> {
        let parts = split_path(path).ok()?;
        match node_at(&self.root, &parts) {
            Ok(Node::File { data }) => std::str::from_utf8(data).ok(),
            _ => None,
        }
    }

    /// File bytes, or `None` if missing or a directory.
    pub fn read_bytes(&self, path: &str) -> Option<&[u8]> {
        let parts = split_path(path).ok()?;
        match node_at(&self.root, &parts) {
            Ok(Node::File { data }) => Some(data.as_slice()),
            _ => None,
        }
    }

    /// Walk directories then files in tree order; file bodies are borrowed.
    /// `content` is `None` for a directory and `Some(bytes)` for a file.
    /// Callers decide which entries are "binary" (e.g. using `kind` predicates).
    pub fn visit_entries(&self, mut visit: impl FnMut(&str, Option<&[u8]>)) {
        visit_entries("", &self.root, &mut visit);
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
        matches!(node_at(&self.root, &parts), Ok(Node::File { .. }))
    }

    /// Immediate children as `(name, is_dir)`, sorted by [`BTreeMap`] order.
    pub fn children<'a>(&'a self, path: &'a str) -> impl Iterator<Item = (&'a str, bool)> + 'a {
        dir_children(&self.root, path)
            .into_iter()
            .flat_map(|map| map.iter())
            .map(|(name, node)| (name.as_str(), matches!(node, Node::Dir { .. })))
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
        Ok(join_path(parent_path(path), new_name))
    }

    /// Copy each `sources` entry into `dest_dir`, cloning nodes first.
    ///
    /// Name clashes get a `unique_name` suffix. Sources are cloned before any
    /// insert so pasting a folder into itself (or a descendant) is safe.
    pub fn copy_entries_into(
        &mut self,
        sources: &[String],
        dest_dir: &str,
    ) -> Result<usize, String> {
        if !self.is_dir(dest_dir) {
            return Err(format!("'{dest_dir}' is not a folder"));
        }
        let mut clones = Vec::new();
        for src in sources {
            clones.push((file_name(src).to_string(), self.clone_node(src)?));
        }
        for (name, node) in clones {
            let dest_name = unique_name(&name, |candidate| {
                self.exists(&join_path(dest_dir, candidate))
            });
            self.insert_node(&join_path(dest_dir, &dest_name), node)?;
        }
        Ok(sources.len())
    }

    fn clone_node(&self, path: &str) -> Result<Node, String> {
        let parts = split_path(path)?;
        if parts.is_empty() {
            return Err("Cannot copy the workspace root".into());
        }
        node_at(&self.root, &parts).cloned()
    }

    fn insert_node(&mut self, path: &str, node: Node) -> Result<(), String> {
        let parts = split_path(path)?;
        let Some((name, parent_parts)) = parts.split_last() else {
            return Err("Cannot write at the workspace root".into());
        };
        let parent = parent_map_mut(&mut self.root, parent_parts)?;
        if parent.contains_key(*name) {
            return Err(format!("'{path}' already exists"));
        }
        parent.insert(name.to_string(), node);
        Ok(())
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
        Node::File { .. } => Err("parent is a file".into()),
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
    fn bytes_round_trip_and_read_file_for_utf8() {
        let mut vfs = Vfs::default();
        // Use bytes that are invalid as UTF-8
        let bin = vec![0xFF, 0xFE, 0x00, 0x01];
        vfs.put_bytes("games/a/face.pdf", bin.clone()).unwrap();
        assert!(vfs.is_file("games/a/face.pdf"));
        assert!(vfs.read_file("games/a/face.pdf").is_none()); // invalid utf8 → not text
        assert_eq!(vfs.read_bytes("games/a/face.pdf"), Some(bin.as_slice()));

        vfs.put_file("games/a/note.txt", "hello".into()).unwrap();
        assert_eq!(vfs.read_file("games/a/note.txt"), Some("hello"));
        assert_eq!(vfs.read_bytes("games/a/note.txt"), Some(b"hello" as &[u8]));
    }

    #[test]
    fn copy_file_into_folder_keeps_source() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/data.json5", "x".into()).unwrap();
        vfs.mkdir("games/b").unwrap();
        let n = vfs
            .copy_entries_into(&["games/a/data.json5".into()], "games/b")
            .unwrap();
        assert_eq!(n, 1);
        assert_eq!(vfs.read_file("games/b/data.json5"), Some("x"));
        assert_eq!(vfs.read_file("games/a/data.json5"), Some("x"));
    }

    #[test]
    fn copy_folder_duplicates_tree() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/n/x.txt", "hi".into()).unwrap();
        vfs.mkdir("games/b").unwrap();
        vfs.copy_entries_into(&["games/a".into()], "games/b")
            .unwrap();
        assert_eq!(vfs.read_file("games/b/a/n/x.txt"), Some("hi"));
        assert_eq!(vfs.read_file("games/a/n/x.txt"), Some("hi"));
    }

    #[test]
    fn copy_name_conflict_gets_suffix() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/f.txt", "1".into()).unwrap();
        vfs.put_file("games/b/f.txt", "2".into()).unwrap();
        vfs.copy_entries_into(&["games/a/f.txt".into()], "games/b")
            .unwrap();
        assert_eq!(vfs.read_file("games/b/f.txt"), Some("2"));
        assert_eq!(vfs.read_file("games/b/f.txt-1"), Some("1"));
    }

    #[test]
    fn copy_binary_data_file() {
        let mut vfs = Vfs::default();
        let bin = vec![0xFF, 0x00, 0xFF];
        vfs.put_bytes("games/a/face.pdf", bin.clone()).unwrap();
        vfs.mkdir("games/b").unwrap();
        vfs.copy_entries_into(&["games/a/face.pdf".into()], "games/b")
            .unwrap();
        assert_eq!(vfs.read_bytes("games/b/face.pdf"), Some(bin.as_slice()));
        assert!(vfs.is_file("games/b/face.pdf"));
        assert!(vfs.read_file("games/b/face.pdf").is_none());
    }

    #[test]
    fn copy_folder_into_itself_nests_a_clone() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/x.txt", "hi".into()).unwrap();
        vfs.copy_entries_into(&["games/a".into()], "games/a")
            .unwrap();
        assert_eq!(vfs.read_file("games/a/x.txt"), Some("hi"));
        assert_eq!(vfs.read_file("games/a/a/x.txt"), Some("hi"));
    }

    #[test]
    fn copy_into_missing_folder_errors() {
        let mut vfs = Vfs::default();
        vfs.put_file("games/a/x.txt", "hi".into()).unwrap();
        let err = vfs
            .copy_entries_into(&["games/a/x.txt".into()], "games/missing")
            .unwrap_err();
        assert!(err.contains("not a folder"), "{err}");
    }

    #[test]
    fn copy_missing_source_errors() {
        let mut vfs = Vfs::default();
        vfs.mkdir("games/b").unwrap();
        let err = vfs
            .copy_entries_into(&["games/nope".into()], "games/b")
            .unwrap_err();
        assert!(err.contains("not found"), "{err}");
    }
}

fn visit_entries(
    prefix: &str,
    children: &BTreeMap<String, Node>,
    visit: &mut impl FnMut(&str, Option<&[u8]>),
) {
    for (name, node) in children {
        let path = join_path(prefix, name);
        match node {
            Node::Dir { children } => {
                visit(&path, None);
                visit_entries(&path, children, visit);
            }
            Node::File { data } => visit(&path, Some(data.as_slice())),
        }
    }
}
