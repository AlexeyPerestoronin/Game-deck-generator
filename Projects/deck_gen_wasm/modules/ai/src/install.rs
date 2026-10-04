//! Install AI files fetched from GitHub `Templates/ai-settings` into the VFS.
//!
//! Mapping is pure: `*.json5` → `ai-models/<name>`, `*.md` → `help/<name>`.
//! Writes only when the path is missing or the stored body is the app HTML shell.

use deck_gen_wasm_conf as wconf;
use deck_gen_wasm_fs::{file_name, Vfs};

/// Map a GitHub blob path under [`wconf::ai::GITHUB_SETTINGS`] to a VFS path.
fn vfs_path_for_remote(remote_path: &str) -> Option<String> {
    let prefix = format!("{}/", wconf::ai::GITHUB_SETTINGS);
    let rest = remote_path.strip_prefix(&prefix)?;
    let name = file_name(rest);
    if name.is_empty() {
        return None;
    }
    if name.ends_with(".json5") {
        Some(format!("{}/{name}", wconf::ai::DIR))
    } else if name.ends_with(".md") {
        Some(format!("{}/{name}", wconf::help::DIR))
    } else {
        None
    }
}

/// True when any non-HTML model json5 already lives under `ai-models/`.
pub fn has_installed_ai_files(vfs: &Vfs) -> bool {
    vfs.children(wconf::ai::DIR).any(|(name, is_dir)| {
        !is_dir
            && name.ends_with(".json5")
            && vfs
                .read_file(&format!("{}/{name}", wconf::ai::DIR))
                .is_some_and(|body| !looks_like_html_document(body))
    })
}

/// Create `ai-models/` (+ `log/`) and write fetched files that still need install.
pub fn install_ai_files(vfs: &mut Vfs, files: &[(String, String)]) -> Result<(), String> {
    vfs.mkdir(wconf::ai::DIR)?;
    vfs.mkdir(wconf::ai::LOG_DIR)?;
    for (remote, body) in files {
        let Some(path) = vfs_path_for_remote(remote) else {
            continue;
        };
        if file_needs_install(vfs, &path) {
            vfs.put_file(&path, body.clone())?;
        }
    }
    Ok(())
}

fn file_needs_install(vfs: &Vfs, path: &str) -> bool {
    match vfs.read_file(path) {
        Some(body) => looks_like_html_document(body),
        None => true,
    }
}

pub(crate) fn looks_like_html_document(body: &str) -> bool {
    let t = body.trim_start();
    let n = t.len().min(32);
    let prefix = t.get(..n).unwrap_or(t).to_ascii_lowercase();
    prefix.starts_with("<!doctype html") || prefix.starts_with("<html")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(name: &str) -> String {
        format!("{}/{name}", wconf::ai::GITHUB_SETTINGS)
    }

    #[test]
    fn maps_json5_to_ai_models_and_md_to_help() {
        assert_eq!(
            vfs_path_for_remote(&remote("gemini-2.0-flash.json5")).as_deref(),
            Some("ai-models/gemini-2.0-flash.json5")
        );
        assert_eq!(
            vfs_path_for_remote(&remote("gemini-3.1-flash-lite.json5")).as_deref(),
            Some("ai-models/gemini-3.1-flash-lite.json5")
        );
        assert_eq!(
            vfs_path_for_remote(&remote("create-game-pt-en.md")).as_deref(),
            Some("help/create-game-pt-en.md")
        );
        assert_eq!(
            vfs_path_for_remote(&remote("edit-game-pt-ru.md")).as_deref(),
            Some("help/edit-game-pt-ru.md")
        );
        assert_eq!(
            vfs_path_for_remote(&remote("ai-help-en.md")).as_deref(),
            Some("help/ai-help-en.md")
        );
        assert_eq!(vfs_path_for_remote(&remote("notes.txt")), None);
        assert_eq!(vfs_path_for_remote("Games/x.json5"), None);
        assert_eq!(
            vfs_path_for_remote(&format!(
                "{}/nested/grok-3-mini.json5",
                wconf::ai::GITHUB_SETTINGS
            ))
            .as_deref(),
            Some("ai-models/grok-3-mini.json5")
        );
    }

    #[test]
    fn missing_files_are_written_and_dirs_created() {
        let mut vfs = Vfs::default();
        assert!(!has_installed_ai_files(&vfs));
        install_ai_files(
            &mut vfs,
            &[
                (
                    remote("gemini-2.0-flash.json5"),
                    "{ id: \"gemini-2.0-flash\" }".into(),
                ),
                (remote("ai-help-en.md"), "# help en\n".into()),
                (remote("ai-help-ru.md"), "# help ru\n".into()),
                (
                    remote("create-game-pt-en.md"),
                    "create en {user_prompt}".into(),
                ),
                (remote("skip.txt"), "nope".into()),
            ],
        )
        .unwrap();
        assert!(vfs.is_file("ai-models/gemini-2.0-flash.json5"));
        assert!(vfs.is_file("help/ai-help-en.md"));
        assert!(vfs.is_file("help/ai-help-ru.md"));
        assert!(vfs.is_file("help/create-game-pt-en.md"));
        assert!(!vfs.is_file("ai-models/skip.txt"));
        assert!(!vfs.is_file("help/skip.txt"));
        assert!(vfs.is_dir(wconf::ai::LOG_DIR));
        assert!(has_installed_ai_files(&vfs));
    }

    #[test]
    fn user_key_is_not_overwritten() {
        let mut vfs = Vfs::default();
        install_ai_files(
            &mut vfs,
            &[(remote("gemini-2.0-flash.json5"), "{ api_key: \"\" }".into())],
        )
        .unwrap();
        vfs.put_file(
            "ai-models/gemini-2.0-flash.json5",
            "{ id: \"gemini-2.0-flash\", api_key: \"SECRET\" }".into(),
        )
        .unwrap();
        install_ai_files(
            &mut vfs,
            &[(remote("gemini-2.0-flash.json5"), "{ overwritten }".into())],
        )
        .unwrap();
        let body = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(body.contains("SECRET"));
        assert!(!body.contains("overwritten"));
    }

    #[test]
    fn custom_markdown_is_not_overwritten() {
        let mut vfs = Vfs::default();
        let path = "help/create-game-pt-en.md";
        vfs.put_file(path, "# my prompt\n".into()).unwrap();
        install_ai_files(
            &mut vfs,
            &[(remote("create-game-pt-en.md"), "bundled".into())],
        )
        .unwrap();
        assert_eq!(vfs.read_file(path), Some("# my prompt\n"));
    }

    #[test]
    fn html_shell_is_reinstalled() {
        let mut vfs = Vfs::default();
        vfs.put_file(
            "ai-models/gemini-2.0-flash.json5",
            "<!DOCTYPE html>\n<html></html>".into(),
        )
        .unwrap();
        assert!(!has_installed_ai_files(&vfs));
        assert!(file_needs_install(&vfs, "ai-models/gemini-2.0-flash.json5"));
        install_ai_files(
            &mut vfs,
            &[(
                remote("gemini-2.0-flash.json5"),
                "{ id: \"gemini-2.0-flash\" }".into(),
            )],
        )
        .unwrap();
        let body = vfs.read_file("ai-models/gemini-2.0-flash.json5").unwrap();
        assert!(body.contains("gemini-2.0-flash"));
        assert!(!looks_like_html_document(body));
        assert!(has_installed_ai_files(&vfs));
    }

    #[test]
    fn html_shell_markdown_is_reinstalled() {
        let mut vfs = Vfs::default();
        let path = "help/ai-help-en.md";
        vfs.put_file(path, "<!DOCTYPE html>\n<html></html>".into())
            .unwrap();
        install_ai_files(&mut vfs, &[(remote("ai-help-en.md"), "# help\n".into())]).unwrap();
        assert_eq!(vfs.read_file(path), Some("# help\n"));
    }
}
