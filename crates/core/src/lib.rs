//! # VanGoghCat Core
//!
//! Domain types, protocol constants, and shared enumerations for the
//! Van Gogh CryptoCats Rust ecosystem. This crate is the single source
//! of truth for all data structures exchanged between subsystems.

pub mod types;
pub mod constants;
pub mod error;
pub mod auction;

pub use types::*;
pub use constants::*;
pub use error::VgcError;
pub use auction::*;
