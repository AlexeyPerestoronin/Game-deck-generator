//! Workspace paths: `/`-separated, no leading slash, no `.` / `..`.
//!
//! The in-memory tree is addressed with POSIX-like strings (`games/foo/data.json5`).
//! These helpers are the only path algebra used by VFS, explorer, and importers.

use std::collections::HashSet;

/// Parent of `path`, or `""` for a top-level name.
pub fn parent_path(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((parent, _)) => parent,
        None => "",
    }
}

/// `parent/name`, or `name` when `parent` is the workspace root.
pub fn join_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// Last `/`-separated segment.
pub fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Extension after the last `.` in the file name; `None` for dotfiles or no ext.
pub(crate) fn file_ext(path: &str) -> Option<&str> {
    let name = file_name(path);
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty() || ext.is_empty() {
        None
    } else {
        Some(ext)
    }
}

/// Split into segments, rejecting `\`, leading `/`, empty parts, `.`, and `..`.
pub(crate) fn split_path(path: &str) -> Result<Vec<&str>, String> {
    if path.is_empty() {
        return Ok(Vec::new());
    }
    if path.starts_with('/') || path.contains('\\') {
        return Err("Use paths like folder/file.txt".into());
    }
    let parts: Vec<&str> = path.split('/').collect();
    let invalid = parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..");
    if invalid {
        return Err(format!("Invalid path '{path}'"));
    }
    Ok(parts)
}

/// True when `path` is `ancestor` or a descendant (`ancestor/...`).
pub fn path_is_or_under(path: &str, ancestor: &str) -> bool {
    path == ancestor
        || path.starts_with(ancestor) && path.as_bytes().get(ancestor.len()) == Some(&b'/')
}

/// If `path` is `old` or lives under it, rewrite the prefix to `new`.
pub fn rewrite_prefix(path: &str, old: &str, new: &str) -> String {
    if path == old {
        return new.to_string();
    }
    let prefix = format!("{old}/");
    match path.strip_prefix(&prefix) {
        Some(rest) => format!("{new}/{rest}"),
        None => path.to_string(),
    }
}

/// Rewrite every path in `set` with [`rewrite_prefix`].
pub fn rewrite_set(set: &mut HashSet<String>, old: &str, new: &str) {
    *set = set
        .iter()
        .map(|current| rewrite_prefix(current, old, new))
        .collect();
}

/// Drop `path` and every descendant from `set`.
pub fn retain_not_under(set: &mut HashSet<String>, path: &str) {
    set.retain(|current| !path_is_or_under(current, path));
}

/// First name in the series `base`, `base-1`, `base-2`, … that `taken` rejects.
pub fn unique_name(base: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(base) {
        return base.to_string();
    }
    let mut n = 1u32;
    loop {
        let name = format!("{base}-{n}");
        if !taken(&name) {
            return name;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_and_parent_roundtrip() {
        let path = join_path("src", "main.rs");
        assert_eq!(path, "src/main.rs");
        assert_eq!(parent_path(&path), "src");
        assert_eq!(file_name(&path), "main.rs");
        assert_eq!(file_ext(&path), Some("rs"));
        assert_eq!(file_ext("help.md"), Some("md"));
        assert_eq!(file_ext("data.json5"), Some("json5"));
        assert_eq!(file_ext(".gitignore"), None);
    }

    #[test]
    fn root_parent_is_empty() {
        assert_eq!(parent_path("README.md"), "");
        assert_eq!(join_path("", "README.md"), "README.md");
    }

    #[test]
    fn rejects_dot_segments() {
        assert!(split_path("../x").is_err());
        assert!(split_path("a//b").is_err());
        assert!(split_path("/abs").is_err());
    }

    #[test]
    fn rewrite_prefix_file_and_children() {
        assert_eq!(rewrite_prefix("a/b", "a/b", "a/c"), "a/c");
        assert_eq!(rewrite_prefix("a/b/d", "a/b", "a/c"), "a/c/d");
        assert_eq!(rewrite_prefix("a/x", "a/b", "a/c"), "a/x");
    }

    #[test]
    fn unique_name_suffixes_when_taken() {
        let taken = |name: &str| name == "demo" || name == "demo-1";
        assert_eq!(unique_name("demo", taken), "demo-2");
        assert_eq!(unique_name("fresh", taken), "fresh");
    }

    #[test]
    fn path_is_or_under_needs_slash_boundary() {
        assert!(path_is_or_under("games/a", "games/a"));
        assert!(path_is_or_under("games/a/b.txt", "games/a"));
        assert!(!path_is_or_under("games/ab", "games/a"));
        assert!(!path_is_or_under("games", "games/a"));
    }

    #[test]
    fn retain_not_under_drops_descendants() {
        let mut set = HashSet::from(["games/a".into(), "games/a/b".into(), "games/c".into()]);
        retain_not_under(&mut set, "games/a");
        assert_eq!(set, HashSet::from(["games/c".into()]));
    }

    #[test]
    fn rewrite_set_rewrites_prefix() {
        let mut set = HashSet::from(["games/a".into(), "games/a/b".into(), "other".into()]);
        rewrite_set(&mut set, "games/a", "games/z");
        assert!(set.contains("games/z"));
        assert!(set.contains("games/z/b"));
        assert!(set.contains("other"));
        assert!(!set.contains("games/a"));
    }
}
