//! Copy a picked folder into `games/` under a unique sibling name.

use crate::conf;
use crate::fs::{join_path, unique_name, Vfs};

/// Unique sibling under `games/`; empty `base` becomes `"game"`.
pub fn unique_folder_name(base: &str, taken: impl Fn(&str) -> bool) -> String {
    let base = if base.is_empty() {
        conf::import::DEFAULT_FOLDER_NAME
    } else {
        base
    };
    unique_name(base, taken)
}

/// Copy `dirs` / `files` into `games/{unique}` and return that folder name.
pub fn install_folder(
    vfs: &mut Vfs,
    name: &str,
    dirs: &[String],
    files: &[(String, String)],
) -> Result<String, String> {
    let folder = unique_folder_name(name, |candidate| vfs.exists(&format!("games/{candidate}")));
    let root = format!("games/{folder}");
    vfs.mkdir(&root)?;
    for dir in dirs {
        vfs.mkdir(&join_path(&root, dir))?;
    }
    for (rel, content) in files {
        vfs.put_file(&join_path(&root, rel), content.clone())?;
    }
    Ok(folder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_folder_gets_suffix() {
        let taken = |name: &str| name == "demo" || name == "demo-1";
        assert_eq!(unique_folder_name("demo", taken), "demo-2");
        assert_eq!(unique_folder_name("fresh", taken), "fresh");
    }
}
