use leptos::prelude::*;

mod app;
mod export;
mod fs;
mod persist;
mod ui;
mod workspace;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
