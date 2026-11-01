//! # VanGoghCat Storage
//!
//! Sled embedded KV database, zero-copy blob engine, and strict 8 GB quota guard.

pub mod db;
pub mod engine;
pub mod quota;

pub use db::StorageDb;
pub use engine::FileStorageEngine;
pub use quota::QuotaGuard;
