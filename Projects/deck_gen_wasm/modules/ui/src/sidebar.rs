//! SidebarMode + pure toggle for the VSCode-style activity bar.
//!
//! The left sidebar is a mode container (Hidden | Explorer tree | Games Local/Global).
//! Toggle is pure and covered by unit tests. Width restore and active styling handled by callers.
//! Not persisted.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SidebarMode {
    #[default]
    Hidden,
    Explorer,
    Games,
}

/// Pure function: clicking the `clicked` button on the activity bar.
/// If the same mode is already active -> Hidden (and width goes to 0 caller).
/// Otherwise switch to the clicked mode (caller ensures non-zero width if needed).
pub fn toggle_sidebar(current: SidebarMode, clicked: SidebarMode) -> SidebarMode {
    if current == clicked {
        SidebarMode::Hidden
    } else {
        clicked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_same_hides() {
        assert_eq!(
            toggle_sidebar(SidebarMode::Explorer, SidebarMode::Explorer),
            SidebarMode::Hidden
        );
        assert_eq!(
            toggle_sidebar(SidebarMode::Games, SidebarMode::Games),
            SidebarMode::Hidden
        );
        assert_eq!(
            toggle_sidebar(SidebarMode::Hidden, SidebarMode::Hidden),
            SidebarMode::Hidden
        );
    }

    #[test]
    fn toggle_other_switches_and_from_hidden() {
        assert_eq!(
            toggle_sidebar(SidebarMode::Hidden, SidebarMode::Explorer),
            SidebarMode::Explorer
        );
        assert_eq!(
            toggle_sidebar(SidebarMode::Hidden, SidebarMode::Games),
            SidebarMode::Games
        );
        assert_eq!(
            toggle_sidebar(SidebarMode::Explorer, SidebarMode::Games),
            SidebarMode::Games
        );
        assert_eq!(
            toggle_sidebar(SidebarMode::Games, SidebarMode::Explorer),
            SidebarMode::Explorer
        );
    }
}
