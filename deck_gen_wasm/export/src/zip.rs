//! Encode the in-memory tree as a ZIP byte buffer.
//!
//! Directories are stored with a trailing `/`; files use DEFLATE. The walk
//! order is the VFS `BTreeMap` order so archives are deterministic for the
//! same tree. File bodies are written from borrowed slices (no extra `Vec`).

use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use deck_gen_wasm_fs::Vfs;

/// Build a ZIP of every directory and file in `vfs`.
pub fn vfs_to_zip(vfs: &Vfs) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut err = None;
    vfs.visit_entries(|path, content| {
        if err.is_some() {
            return;
        }
        let result = match content {
            None => zip.add_directory(format!("{path}/"), options),
            Some(bytes) => zip
                .start_file(path, options)
                .and_then(|_| zip.write_all(bytes).map_err(Into::into)),
        };
        if let Err(e) = result {
            err = Some(e.to_string());
        }
    });
    if let Some(err) = err {
        return Err(err);
    }
    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}
