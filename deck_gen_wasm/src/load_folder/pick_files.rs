//! Browser multi-file picker: hidden `<input type="file" multiple>`.
//!
//! Accepts the text extensions in [`crate::conf::import::ALLOWED_EXTENSIONS`]
//! plus image types. Oversized images and other extensions reject the batch.

use super::input::pick_with_hidden_input;
use super::policy::file_input_accept;
use super::read::collect_picked_files;
use super::PickFilesResult;

/// Open a file picker and read allowed files (or a reject/cancel).
pub async fn pick_and_read_files() -> PickFilesResult {
    match pick_with_hidden_input(|input| {
        input.set_multiple(true);
        input.set_accept(&file_input_accept());
    })
    .await
    {
        None => PickFilesResult::Cancelled,
        Some(list) => match collect_picked_files(list).await {
            Ok(entries) => match entries.reject_reason() {
                Some(reason) => PickFilesResult::Rejected(reason),
                None => PickFilesResult::Ready {
                    files: entries.files,
                },
            },
            Err(err) => PickFilesResult::Rejected(err),
        },
    }
}
