//! Planned copy set for the explorer (not the OS clipboard).
//!
//! Choosing `copy` only marks paths. Paste later clones those VFS nodes into
//! the target folder. Nested marks are dropped so a folder and a child inside
//! it are not pasted twice.

use std::collections::HashSet;

use crate::fs::parent_path;

/// True when `path` itself or an ancestor folder is in `selected`.
pub fn row_looks_selected(path: &str, selected: &HashSet<String>) -> bool {
    if selected.contains(path) {
        return true;
    }
    let mut current = parent_path(path);
    while !current.is_empty() {
        if selected.contains(&current) {
            return true;
        }
        current = parent_path(&current);
    }
    false
}

/// Drop paths whose ancestor is also in `paths` (sorted for stable paste order).
pub fn top_level_paths(paths: &HashSet<String>) -> Vec<String> {
    let mut items: Vec<String> = paths.iter().cloned().collect();
    items.sort();
    items
        .into_iter()
        .filter(|path| {
            let parent = parent_path(path);
            parent.is_empty() || !row_looks_selected(&parent, paths)
        })
        .collect()
}

/// Method #2: mark the whole selection. Method #1: toggle the clicked path.
pub fn apply_copy_command(
    clicked: &str,
    selected: &HashSet<String>,
    planned: &HashSet<String>,
) -> HashSet<String> {
    let mut next = planned.clone();
    if selected.contains(clicked) && selected.len() > 1 {
        for path in selected {
            next.insert(path.clone());
        }
    } else if !next.remove(clicked) {
        next.insert(clicked.to_string());
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_selection_covers_children() {
        let selected = HashSet::from(["games/a".into()]);
        assert!(row_looks_selected("games/a", &selected));
        assert!(row_looks_selected("games/a/b.txt", &selected));
        assert!(row_looks_selected("games/a/n/c.txt", &selected));
        assert!(!row_looks_selected("games/b", &selected));
        assert!(!row_looks_selected("games", &selected));
    }

    #[test]
    fn toggle_copy_on_single_path() {
        let selected = HashSet::new();
        let planned = HashSet::new();
        let planned = apply_copy_command("a/b", &selected, &planned);
        assert!(planned.contains("a/b"));
        let planned = apply_copy_command("a/b", &selected, &planned);
        assert!(planned.is_empty());
    }

    #[test]
    fn copy_marks_whole_selection() {
        let selected = HashSet::from(["a".into(), "b".into()]);
        let planned = apply_copy_command("a", &selected, &HashSet::new());
        assert!(planned.contains("a"));
        assert!(planned.contains("b"));
        let planned = apply_copy_command("a", &selected, &planned);
        assert_eq!(planned.len(), 2);
    }

    #[test]
    fn single_selected_item_still_toggles() {
        let selected = HashSet::from(["a".into()]);
        let planned = apply_copy_command("a", &selected, &HashSet::new());
        assert_eq!(planned.len(), 1);
        let planned = apply_copy_command("a", &selected, &planned);
        assert!(planned.is_empty());
    }

    #[test]
    fn top_level_drops_nested() {
        let paths = HashSet::from(["games/a".into(), "games/a/b".into(), "games/c".into()]);
        assert_eq!(
            top_level_paths(&paths),
            vec!["games/a".to_string(), "games/c".to_string()]
        );
    }
}
