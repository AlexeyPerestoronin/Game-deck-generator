//! Walk a directory handle or `FileList` into relative paths and file bodies.
//!
//! The File System Access tree is an async iterator of `[name, handle]` pairs.
//! The `<input webkitdirectory>` path is a flat list with `webkitRelativePath`.
//! Classification (extension / size) is synchronous; allowed files are then
//! read in a bounded parallel join.

use std::collections::BTreeSet;

use js_sys::{Array, Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileList};

use super::policy::{classify, ImportClass};
use super::FileBody;
use crate::conf;
use crate::fs::parent_path;
use crate::js;

/// Files, directories, extension rejects, and oversized images from one walk.
pub(super) struct CollectedEntries {
    pub files: Vec<(String, FileBody)>,
    pub dirs: Vec<String>,
    pub rejected: Vec<String>,
    pub oversized: Vec<String>,
}

impl CollectedEntries {
    /// Error text when any path was blocked; `None` if the batch is clean.
    pub(super) fn reject_reason(&self) -> Option<String> {
        if self.rejected.is_empty() && self.oversized.is_empty() {
            None
        } else {
            Some(super::policy::import_block_message(
                &self.rejected,
                &self.oversized,
            ))
        }
    }
}

/// Folder name plus [`CollectedEntries`] from a `webkitdirectory` file list.
pub(super) struct CollectedFolder {
    pub name: String,
    pub entries: CollectedEntries,
}

struct PendingRead {
    path: String,
    file: File,
    class: ImportClass,
}

pub(super) fn split_webkit_path(path: &str) -> (Option<String>, String) {
    let path = path.replace('\\', "/");
    match path.split_once('/') {
        Some((root, rest)) if !root.is_empty() && !rest.is_empty() => {
            (Some(root.to_string()), rest.to_string())
        }
        _ => (None, path),
    }
}

pub(super) async fn collect_from_file_list(list: FileList) -> Result<CollectedFolder, String> {
    let mut folder_name = conf::import::DEFAULT_FOLDER_NAME.to_string();
    let mut pending = Vec::new();
    let mut dirs = BTreeSet::new();
    let mut rejected = Vec::new();
    let mut oversized = Vec::new();
    for index in 0..list.length() {
        let Some(file) = list.item(index) else {
            continue;
        };
        let relative = webkit_relative_path(&file);
        let (root, rel) = split_webkit_path(&relative);
        if let Some(root) = root {
            folder_name = root;
        }
        let mut parent = parent_path(&rel);
        while !parent.is_empty() {
            dirs.insert(parent.to_string());
            parent = parent_path(parent);
        }
        classify_file(&mut pending, &mut rejected, &mut oversized, file, rel);
    }
    Ok(CollectedFolder {
        name: folder_name,
        entries: CollectedEntries {
            files: read_pending(pending).await?,
            dirs: dirs.into_iter().collect(),
            rejected,
            oversized,
        },
    })
}

fn webkit_relative_path(file: &File) -> String {
    Reflect::get(file, &JsValue::from_str("webkitRelativePath"))
        .ok()
        .and_then(|value| value.as_string())
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| file.name())
}

/// Read allowed files from a flat (non-directory) `FileList`.
pub(super) async fn collect_picked_files(list: FileList) -> Result<CollectedEntries, String> {
    let mut pending = Vec::new();
    let mut rejected = Vec::new();
    let mut oversized = Vec::new();
    for index in 0..list.length() {
        let Some(file) = list.item(index) else {
            continue;
        };
        let name = file.name();
        classify_file(&mut pending, &mut rejected, &mut oversized, file, name);
    }
    Ok(CollectedEntries {
        files: read_pending(pending).await?,
        dirs: Vec::new(),
        rejected,
        oversized,
    })
}

fn classify_file(
    pending: &mut Vec<PendingRead>,
    rejected: &mut Vec<String>,
    oversized: &mut Vec<String>,
    file: File,
    path: String,
) {
    match classify(&path, file.size() as u64) {
        ImportClass::Rejected => rejected.push(path),
        ImportClass::Oversized => oversized.push(path),
        class @ (ImportClass::Image | ImportClass::Text) => {
            pending.push(PendingRead { path, file, class });
        }
    }
}

async fn read_pending(pending: Vec<PendingRead>) -> Result<Vec<(String, FileBody)>, String> {
    let results = crate::task::map_join(pending, conf::io::FILE_READ_PARALLEL, |item| async move {
        match item.class {
            ImportClass::Image => read_file_bytes(&item.file)
                .await
                .map(|bytes| (item.path, FileBody::Bytes(bytes))),
            ImportClass::Text => read_file_text(&item.file)
                .await
                .map(|text| (item.path, FileBody::Text(text))),
            ImportClass::Rejected | ImportClass::Oversized => {
                Err("internal: classified file was not readable".to_string())
            }
        }
    })
    .await;
    results.into_iter().collect()
}

async fn read_file_text(file: &File) -> Result<String, String> {
    let value = JsFuture::from(file.text())
        .await
        .map_err(|_| "Could not read file".to_string())?;
    value
        .as_string()
        .ok_or_else(|| "Could not read file as text".to_string())
}

async fn read_file_bytes(file: &File) -> Result<Vec<u8>, String> {
    let buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|_| "Could not read file".to_string())?;
    let array = Uint8Array::new(&buffer);
    let mut bytes = vec![0u8; array.length() as usize];
    array.copy_to(&mut bytes);
    Ok(bytes)
}

pub(super) async fn collect_tree(dir: &JsValue, prefix: &str) -> Result<CollectedEntries, String> {
    let mut pending = Vec::new();
    let mut dirs = Vec::new();
    let mut rejected = Vec::new();
    let mut oversized = Vec::new();
    collect_tree_into(
        dir,
        prefix,
        &mut pending,
        &mut dirs,
        &mut rejected,
        &mut oversized,
    )
    .await?;
    Ok(CollectedEntries {
        files: read_pending(pending).await?,
        dirs,
        rejected,
        oversized,
    })
}

async fn collect_tree_into(
    dir: &JsValue,
    prefix: &str,
    pending: &mut Vec<PendingRead>,
    dirs: &mut Vec<String>,
    rejected: &mut Vec<String>,
    oversized: &mut Vec<String>,
) -> Result<(), String> {
    let entries = js::call0(dir, "entries")?;
    loop {
        let next = js::call_async(&entries, "next").await?;
        let done = Reflect::get(&next, &JsValue::from_str("done"))
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(true);
        if done {
            break;
        }
        let value = Reflect::get(&next, &JsValue::from_str("value"))
            .map_err(|_| "folder entry is missing".to_string())?;
        let pair = value
            .dyn_into::<Array>()
            .map_err(|_| "folder entry is not a pair".to_string())?;
        let child_name = pair
            .get(0)
            .as_string()
            .ok_or_else(|| "folder entry has no name".to_string())?;
        if child_name == "." || child_name == ".." {
            continue;
        }
        let child = pair.get(1);
        let kind = Reflect::get(&child, &JsValue::from_str("kind"))
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_default();
        let rel = if prefix.is_empty() {
            child_name.clone()
        } else {
            format!("{prefix}/{child_name}")
        };
        if kind == "directory" {
            dirs.push(rel.clone());
            Box::pin(collect_tree_into(
                &child, &rel, pending, dirs, rejected, oversized,
            ))
            .await?;
        } else {
            let file_val = js::call_async(&child, "getFile").await?;
            let file: File = file_val
                .dyn_into()
                .map_err(|_| format!("Could not read {rel}"))?;
            classify_file(pending, rejected, oversized, file, rel);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_webkit_relative_path() {
        assert_eq!(
            split_webkit_path("demo/decks/data.json5"),
            (Some("demo".into()), "decks/data.json5".into())
        );
        assert_eq!(split_webkit_path("help.md"), (None, "help.md".into()));
    }
}
