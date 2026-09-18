//! localStorage session and IndexedDB binaries.

mod binaries;
mod session;

pub use crate::binaries::{
    binaries_fingerprint, encode_binaries, load_binaries, save_binaries, save_encoded,
};
pub use crate::session::{load_session, save_session, Session};
