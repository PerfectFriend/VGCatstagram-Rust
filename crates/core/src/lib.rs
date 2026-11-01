//! # VanGoghCat Core
//!
//! Domain types, protocol constants, and shared enumerations for the
//! Van Gogh CryptoCats Rust ecosystem. This crate is the single source
//! of truth for all data structures exchanged between subsystems.

pub mod auction;
pub mod config;
pub mod constants;
pub mod error;
pub mod types;

pub use auction::*;
pub use config::VgcConfig;
pub use constants::*;
pub use error::VgcError;
pub use types::*;
