//! # VanGoghCat Storage
//!
//! Sled embedded KV database, zero-copy blob engine, and strict 8 GB quota guard.

pub mod db;
pub mod quota;
pub mod engine;

pub use db::StorageDb;
pub use quota::QuotaGuard;
pub use engine::FileStorageEngine;
