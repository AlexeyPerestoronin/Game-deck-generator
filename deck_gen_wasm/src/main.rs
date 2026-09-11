//! Browser entry for the in-memory game workspace editor.
//!
//! Installs the panic hook, then mounts [`app::App`] as a Leptos CSR root.
//! All filesystem work stays in the VFS; HTML generation is `deck_gen` with
//! `--no-default-features`.

use leptos::prelude::*;

mod app;
mod conf;
mod export;
mod fs;
mod github;
mod http;
mod load_folder;
mod persist;
mod template;
mod ui;
mod workspace;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
