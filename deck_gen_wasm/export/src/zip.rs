//! Encode the in-memory tree as a ZIP byte buffer.
//!
//! Directories are stored with a trailing `/`; files use DEFLATE. The walk
//! order is the VFS `BTreeMap` order so archives are deterministic for the
//! same tree. File bodies are written from borrowed slices (no extra `Vec`).

use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use deck_gen_wasm_fs::Vfs;
use deck_gen_wasm_progress::{progress_loop, progress_wrapper, Progress};

/// Build a ZIP of every directory and file in `vfs`.
pub async fn vfs_to_zip(vfs: &Vfs, progress: Progress) -> Result<Vec<u8>, String> {
    let mut entries: Vec<(String, bool)> = Vec::new();
    vfs.visit_entries(|path, content| {
        entries.push((path.to_string(), content.is_none()));
    });

    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut err = None;
    progress_wrapper!(progress, {
        progress_loop!(progress, 0.0, 100.0, entries, |(path, is_dir)| {
            if err.is_some() {
                continue;
            }
            let result = if is_dir {
                zip.add_directory(format!("{path}/"), options)
            } else {
                match vfs.read_bytes(&path) {
                    Some(bytes) => zip
                        .start_file(&path, options)
                        .and_then(|_| zip.write_all(bytes).map_err(Into::into)),
                    None => {
                        err = Some(format!("missing file {path}"));
                        continue;
                    }
                }
            };
            if let Err(e) = result {
                err = Some(e.to_string());
            }
        });
    });
    if let Some(err) = err {
        return Err(err);
    }
    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn recorder() -> (Progress, Rc<RefCell<Vec<f32>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = Rc::clone(&log);
        (Progress::new(move |pct| log2.borrow_mut().push(pct)), log)
    }

    #[test]
    fn zip_progress_spreads_over_entries() {
        let mut vfs = Vfs::default();
        vfs.put_file("a.txt", "A".into()).unwrap();
        vfs.put_file("b.txt", "B".into()).unwrap();
        let (progress, log) = recorder();
        let bytes = deck_gen_wasm_progress::poll_now(vfs_to_zip(&vfs, progress)).unwrap();
        assert!(!bytes.is_empty());
        // wrapper 0, loop i=0 → 0, i=1 → 50, loop end 100, wrapper 100
        assert_eq!(*log.borrow(), vec![0.0, 0.0, 50.0, 100.0, 100.0]);
    }

    #[test]
    fn empty_vfs_only_bookends() {
        let vfs = Vfs::default();
        let (progress, log) = recorder();
        let _ = deck_gen_wasm_progress::poll_now(vfs_to_zip(&vfs, progress)).unwrap();
        assert_eq!(*log.borrow(), vec![0.0, 100.0, 100.0]);
    }
}
