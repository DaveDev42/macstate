//! Raw macOS FFI used by `macstate-core`.
//!
//! Hand-written, intentionally tiny: only the symbols `macstate-core`
//! actually consumes. Lets downstream consumers (e.g. Tauri apps) avoid
//! pulling in the full objc2 framework crate set.

#![cfg(target_os = "macos")]
#![allow(non_upper_case_globals, non_snake_case)]

pub mod cf;
pub mod dispatch;
pub mod iokit;
pub mod nwpath;
