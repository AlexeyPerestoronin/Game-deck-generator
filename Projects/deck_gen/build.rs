//! Build script for deck_gen.
//
//! The automatic exe copy to repo root has been removed (per refactoring).
//! `start.bat` performs an explicit copy after `cargo build --release -p deck_gen --features cli`.
//! This keeps the crate itself free of post-build side effects.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
