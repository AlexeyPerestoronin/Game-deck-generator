//! Ignored live Q&A against model json5. Not part of the WASM app.
//!
//! From the workspace root (`projects/`):
//! `cargo test -p deck_gen_wasm_ai_tests -- --ignored --nocapture --test-threads=1`

#[cfg(test)]
mod live;
