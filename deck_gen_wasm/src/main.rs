//! Browser entry for the in-memory game workspace editor.
//!
//! Installs the panic hook, then mounts [`deck_gen_wasm_ui::App`] as a Leptos
//! CSR root. Library crates under this folder link into one WASM module.

use leptos::prelude::*;

use deck_gen_wasm_ui::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
