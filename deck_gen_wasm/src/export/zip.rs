//! Encode the in-memory tree as a ZIP byte buffer.
//!
//! Directories are stored with a trailing `/`; files use DEFLATE. The walk
//! order is the VFS `BTreeMap` order so archives are deterministic for the
//! same tree.

use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::fs::Vfs;

/// Build a ZIP of every directory and file in `vfs`.
pub fn vfs_to_zip(vfs: &Vfs) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let (dirs, files) = vfs.files_and_dirs();
    for dir in dirs {
        zip.add_directory(format!("{dir}/"), options)
            .map_err(|err| err.to_string())?;
    }
    for (path, content) in files {
        zip.start_file(path, options)
            .map_err(|err| err.to_string())?;
        zip.write_all(content.as_bytes())
            .map_err(|err| err.to_string())?;
    }
    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}
