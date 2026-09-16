//! Copy a picked folder or files into the in-memory workspace.

use deck_gen_wasm_conf as conf;
use deck_gen_wasm_fs::{file_name, join_path, unique_name, Vfs};
use deck_gen_wasm_progress::{progress_block, progress_loop, progress_wrapper, Progress};

use super::FileBody;

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
pub async fn install_folder(
    vfs: &mut Vfs,
    name: &str,
    dirs: &[String],
    files: &[(String, FileBody)],
    progress: Progress,
) -> Result<String, String> {
    let folder = unique_folder_name(name, |candidate| vfs.exists(&format!("games/{candidate}")));
    let root = format!("games/{folder}");
    progress_wrapper!(progress, {
        progress_block!(progress, 0.0, 10.0, {
            vfs.mkdir(&root)?;
        });
        progress_loop!(progress, 10.0, 40.0, dirs, |dir| {
            vfs.mkdir(&join_path(&root, dir))?;
        });
        progress_loop!(progress, 40.0, 100.0, files, |item| {
            let (rel, body) = item;
            put_body(vfs, &join_path(&root, rel), body)?;
        });
    });
    Ok(folder)
}

/// Copy `files` into an existing workspace `folder` (file name only, no subdirs).
pub async fn install_files(
    vfs: &mut Vfs,
    folder: &str,
    files: &[(String, FileBody)],
    progress: Progress,
) -> Result<usize, String> {
    if !vfs.is_dir(folder) {
        return Err(format!("'{folder}' is not a folder"));
    }
    progress_wrapper!(progress, {
        progress_loop!(progress, 0.0, 100.0, files, |item| {
            let (name, body) = item;
            let dest = join_path(folder, file_name(name));
            put_body(vfs, &dest, body)?;
        });
    });
    Ok(files.len())
}

fn put_body(vfs: &mut Vfs, path: &str, body: &FileBody) -> Result<(), String> {
    match body {
        FileBody::Text(content) => vfs.put_file(path, content.clone()),
        FileBody::Bytes(data) => vfs.put_bytes(path, data.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn silent() -> Progress {
        Progress::new(|_| {})
    }

    #[test]
    fn unique_folder_gets_suffix() {
        let taken = |name: &str| name == "demo" || name == "demo-1";
        assert_eq!(unique_folder_name("demo", taken), "demo-2");
        assert_eq!(unique_folder_name("fresh", taken), "fresh");
    }

    #[test]
    fn install_folder_writes_text_and_image() {
        let mut vfs = Vfs::default();
        let folder = deck_gen_wasm_progress::poll_now(install_folder(
            &mut vfs,
            "demo",
            &["art".into()],
            &[
                ("help.md".into(), FileBody::Text("# hi".into())),
                ("art/logo.png".into(), FileBody::Bytes(vec![0x89, 0x50])),
            ],
            silent(),
        ))
        .unwrap();
        assert_eq!(folder, "demo");
        assert_eq!(vfs.read_file("games/demo/help.md"), Some("# hi"));
        assert!(deck_gen_wasm_fs::kind::is_image("games/demo/art/logo.png"));
        assert_eq!(
            vfs.read_bytes("games/demo/art/logo.png"),
            Some(&[0x89, 0x50][..])
        );
    }

    #[test]
    fn install_files_into_existing_folder() {
        let mut vfs = Vfs::default();
        vfs.mkdir("games/demo").unwrap();
        let n = deck_gen_wasm_progress::poll_now(install_files(
            &mut vfs,
            "games/demo",
            &[("logo.png".into(), FileBody::Bytes(vec![1, 2, 3]))],
            silent(),
        ))
        .unwrap();
        assert_eq!(n, 1);
        assert!(vfs.is_file("games/demo/logo.png"));
    }
}
