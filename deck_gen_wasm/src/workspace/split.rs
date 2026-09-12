//! Split-for-preview: two editor panes and the tab-routing that feeds them.
//!
//! The workspace has one tab list until the activity-bar toggle is on. Then
//! edit tabs stay on the left and preview tabs move to the right. Opening a
//! previewable file while split also opens its preview tab if it is missing.
//! Partition / merge / auto-open rules are pure functions so they can be
//! unit-tested without Leptos; [`Workspace`] methods apply them to signals.

use leptos::prelude::*;

use super::{OpenTab, TabKind, Workspace};
use crate::fs::file_ext;
use crate::load_folder::is_image;

/// HTML, Markdown, PDF, and images can render in a preview tab.
pub(crate) fn is_previewable(path: &str) -> bool {
    if is_image(path) {
        return true;
    }
    matches!(
        file_ext(path).map(str::to_ascii_lowercase).as_deref(),
        Some("html" | "htm" | "md" | "markdown" | "pdf")
    )
}

/// Split a mixed tab bar into edit (left) and preview (right) lists.
fn partition_tabs(tabs: Vec<OpenTab>) -> (Vec<OpenTab>, Vec<OpenTab>) {
    let mut edit = Vec::new();
    let mut preview = Vec::new();
    for tab in tabs {
        match tab.kind {
            TabKind::Edit => edit.push(tab),
            TabKind::Preview => preview.push(tab),
        }
    }
    (edit, preview)
}

/// Put left-pane tabs back first, then right-pane tabs that are not duplicates.
fn merge_split_tabs(left: Vec<OpenTab>, right: Vec<OpenTab>) -> Vec<OpenTab> {
    let mut merged = left;
    for tab in right {
        if !merged.contains(&tab) {
            merged.push(tab);
        }
    }
    merged
}

/// Left/right active tabs after turning split on.
///
/// The previous active tab stays in its pane; the other pane uses its first tab.
fn active_after_split(
    previous: Option<&OpenTab>,
    edit: &[OpenTab],
    preview: &[OpenTab],
) -> (Option<OpenTab>, Option<OpenTab>) {
    let left = match previous {
        Some(tab) if tab.kind == TabKind::Edit => Some(tab.clone()),
        _ => edit.first().cloned(),
    };
    let right = match previous {
        Some(tab) if tab.kind == TabKind::Preview => Some(tab.clone()),
        _ => preview.first().cloned(),
    };
    (left, right)
}

/// Active tab after turning split off: keep the left active tab if it remains.
fn active_after_merge(left_active: Option<&OpenTab>, merged: &[OpenTab]) -> Option<OpenTab> {
    left_active
        .filter(|tab| merged.contains(tab))
        .cloned()
        .or_else(|| merged.first().cloned())
}

/// True when split mode should create a preview tab for `path`.
fn should_auto_open_preview(path: &str, preview_tabs: &[OpenTab]) -> bool {
    is_previewable(path)
        && !preview_tabs
            .iter()
            .any(|tab| tab.kind == TabKind::Preview && tab.path == path)
}

/// Remove `tab` and pick the neighbor to the right (else left) when it was active.
fn close_tab_in(
    tabs: Vec<OpenTab>,
    active: Option<OpenTab>,
    tab: &OpenTab,
) -> (Vec<OpenTab>, Option<OpenTab>, bool) {
    let idx = tabs.iter().position(|open| open == tab);
    let remaining: Vec<_> = tabs.into_iter().filter(|open| open != tab).collect();
    let was_active = active.as_ref() == Some(tab);
    if !was_active {
        return (remaining, active, false);
    }
    let next = idx.and_then(|i| {
        if i < remaining.len() {
            remaining.get(i).cloned()
        } else {
            remaining.last().cloned()
        }
    });
    (remaining, next, true)
}

/// Drop tabs whose path (or a prefix of it) matches a deleted entry.
pub(crate) fn forget_tabs(
    tabs: Vec<OpenTab>,
    active: Option<OpenTab>,
    gone: impl Fn(&str) -> bool,
) -> (Vec<OpenTab>, Option<OpenTab>) {
    let closing_active = active.as_ref().is_some_and(|tab| gone(&tab.path));
    let idx = active
        .as_ref()
        .and_then(|tab| tabs.iter().position(|open| open == tab));
    let remaining: Vec<_> = tabs.into_iter().filter(|tab| !gone(&tab.path)).collect();
    let next = if closing_active {
        idx.and_then(|i| {
            if remaining.is_empty() {
                None
            } else {
                remaining.get(i.min(remaining.len() - 1)).cloned()
            }
        })
    } else {
        active.filter(|tab| remaining.contains(tab))
    };
    (remaining, next)
}

impl Workspace {
    /// Toggle the two-pane editor: previews on the right, files on the left.
    pub fn toggle_split_preview(&self) {
        if self.split_preview.get() {
            let left = self.tabs.get();
            let right = self.preview_tabs.get();
            let left_active = self.active_tab.get();
            let merged = merge_split_tabs(left, right);
            let active = active_after_merge(left_active.as_ref(), &merged);
            self.tabs.set(merged);
            self.preview_tabs.set(Vec::new());
            self.active_preview_tab.set(None);
            self.active_tab.set(active.clone());
            self.split_preview.set(false);
            if let Some(tab) = active {
                self.set_primary_selection(Some(tab.path));
            }
        } else {
            let previous = self.active_tab.get();
            let (edit, preview) = partition_tabs(self.tabs.get());
            let (left_active, right_active) =
                active_after_split(previous.as_ref(), &edit, &preview);
            self.tabs.set(edit);
            self.preview_tabs.set(preview);
            self.active_tab.set(left_active);
            self.active_preview_tab.set(right_active);
            self.split_preview.set(true);
        }
    }

    /// Focus `tab`, creating it if it is not already open.
    ///
    /// While split is on, preview tabs go to the right pane. Opening an edit
    /// tab for a previewable file also opens its preview on the right when
    /// that preview tab is not already there.
    pub fn open_tab(&self, tab: OpenTab) {
        let split = self.split_preview.get();
        if split && tab.kind == TabKind::Preview {
            self.push_tab(self.preview_tabs, tab.clone());
            self.active_preview_tab.set(Some(tab));
            return;
        }
        let auto_preview = split
            && tab.kind == TabKind::Edit
            && should_auto_open_preview(&tab.path, &self.preview_tabs.get());
        let preview_path = tab.path.clone();
        self.push_tab(self.tabs, tab.clone());
        self.active_tab.set(Some(tab));
        if auto_preview {
            let preview = OpenTab {
                path: preview_path,
                kind: TabKind::Preview,
            };
            self.push_tab(self.preview_tabs, preview.clone());
            self.active_preview_tab.set(Some(preview));
        }
    }

    fn push_tab(&self, slot: RwSignal<Vec<OpenTab>>, tab: OpenTab) {
        slot.update(move |tabs| {
            if !tabs.contains(&tab) {
                tabs.push(tab);
            }
        });
    }

    /// Make an existing tab active and select its path in the explorer.
    pub fn activate_tab(&self, tab: OpenTab) {
        if self.split_preview.get() && tab.kind == TabKind::Preview {
            self.active_preview_tab.set(Some(tab.clone()));
        } else {
            self.active_tab.set(Some(tab.clone()));
        }
        self.set_primary_selection(Some(tab.path));
    }

    /// Close `tab`. If it was active in its pane, activate the neighbor to the right (else left).
    pub fn close_tab(&self, tab: OpenTab) {
        let preview_pane = self.split_preview.get() && tab.kind == TabKind::Preview;
        let (tabs, active, was_active) = if preview_pane {
            close_tab_in(self.preview_tabs.get(), self.active_preview_tab.get(), &tab)
        } else {
            close_tab_in(self.tabs.get(), self.active_tab.get(), &tab)
        };
        if preview_pane {
            self.preview_tabs.set(tabs);
            if was_active {
                self.active_preview_tab.set(active.clone());
            }
        } else {
            self.tabs.set(tabs);
            if was_active {
                self.active_tab.set(active.clone());
            }
        }
        if was_active {
            self.set_primary_selection(active.map(|tab| tab.path));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(path: &str) -> OpenTab {
        OpenTab {
            path: path.to_string(),
            kind: TabKind::Edit,
        }
    }

    fn preview(path: &str) -> OpenTab {
        OpenTab {
            path: path.to_string(),
            kind: TabKind::Preview,
        }
    }

    #[test]
    fn previewable_html_md_pdf_images_only() {
        assert!(is_previewable("a/preview.html"));
        assert!(is_previewable("help.MD"));
        assert!(is_previewable("doc.markdown"));
        assert!(is_previewable("a/face.pdf"));
        assert!(is_previewable("a/logo.png"));
        assert!(is_previewable("a/mark.JPG"));
        assert!(is_previewable("a/app.icon"));
        assert!(!is_previewable("a/data.json5"));
        assert!(!is_previewable("a/style.scss"));
    }

    #[test]
    fn partition_keeps_order_in_each_pane() {
        let tabs = vec![
            edit("a.md"),
            preview("a.md"),
            edit("b.json5"),
            preview("c.html"),
        ];
        let (left, right) = partition_tabs(tabs);
        assert_eq!(left, vec![edit("a.md"), edit("b.json5")]);
        assert_eq!(right, vec![preview("a.md"), preview("c.html")]);
    }

    #[test]
    fn merge_puts_left_first_then_right() {
        let merged = merge_split_tabs(
            vec![edit("a.md"), edit("b.json5")],
            vec![preview("a.md"), preview("c.html")],
        );
        assert_eq!(
            merged,
            vec![
                edit("a.md"),
                edit("b.json5"),
                preview("a.md"),
                preview("c.html"),
            ]
        );
    }

    #[test]
    fn merge_skips_duplicate_tabs() {
        let tab = edit("a.md");
        let merged = merge_split_tabs(vec![tab.clone()], vec![tab.clone(), preview("a.md")]);
        assert_eq!(merged, vec![edit("a.md"), preview("a.md")]);
    }

    #[test]
    fn split_keeps_edit_active_on_left() {
        let edit_a = edit("a.md");
        let preview_a = preview("a.md");
        let (left, right) =
            active_after_split(Some(&edit_a), &[edit_a.clone()], &[preview_a.clone()]);
        assert_eq!(left, Some(edit_a));
        assert_eq!(right, Some(preview_a));
    }

    #[test]
    fn split_keeps_preview_active_on_right() {
        let edit_a = edit("a.md");
        let preview_a = preview("a.md");
        let (left, right) =
            active_after_split(Some(&preview_a), &[edit_a.clone()], &[preview_a.clone()]);
        assert_eq!(left, Some(edit_a));
        assert_eq!(right, Some(preview_a));
    }

    #[test]
    fn split_empty_preview_pane_when_none_open() {
        let edit_a = edit("a.md");
        let (left, right) = active_after_split(Some(&edit_a), &[edit_a.clone()], &[]);
        assert_eq!(left, Some(edit_a));
        assert_eq!(right, None);
    }

    #[test]
    fn merge_keeps_left_active() {
        let left_active = edit("b.json5");
        let merged = vec![edit("a.md"), left_active.clone(), preview("a.md")];
        assert_eq!(
            active_after_merge(Some(&left_active), &merged),
            Some(left_active)
        );
    }

    #[test]
    fn auto_open_only_when_previewable_and_missing() {
        assert!(should_auto_open_preview("a.md", &[]));
        assert!(!should_auto_open_preview("a.json5", &[]));
        assert!(!should_auto_open_preview("a.md", &[preview("a.md")]));
        assert!(should_auto_open_preview("a.md", &[edit("a.md")]));
    }

    #[test]
    fn close_active_picks_right_neighbor_then_left() {
        let a = edit("a");
        let b = edit("b");
        let c = edit("c");
        let tabs = vec![a.clone(), b.clone(), c.clone()];
        let (rest, next, was_active) = close_tab_in(tabs.clone(), Some(b.clone()), &b);
        assert!(was_active);
        assert_eq!(rest, vec![a.clone(), c.clone()]);
        assert_eq!(next, Some(c.clone()));

        let (rest, next, _) = close_tab_in(vec![a.clone(), b.clone()], Some(b.clone()), &b);
        assert_eq!(rest, vec![a.clone()]);
        assert_eq!(next, Some(a));

        let (rest, next, was_active) = close_tab_in(tabs, Some(c.clone()), &edit("a"));
        assert!(!was_active);
        assert_eq!(next, Some(c));
        assert_eq!(rest.len(), 2);
    }

    #[test]
    fn forget_drops_path_and_children() {
        let tabs = vec![
            edit("games/a.md"),
            preview("games/a.md"),
            edit("games/b/c.html"),
            edit("other.md"),
        ];
        let gone = |path: &str| path == "games" || path.starts_with("games/");
        let (rest, next) = forget_tabs(tabs, Some(edit("games/a.md")), gone);
        assert_eq!(rest, vec![edit("other.md")]);
        assert_eq!(next, Some(edit("other.md")));
    }
}
