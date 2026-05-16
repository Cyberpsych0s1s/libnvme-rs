//! Safe, idiomatic Rust bindings for the Linux `libnvme` C library.

mod error;
mod root;

pub use error::{Error, Result};
pub use root::Root;
