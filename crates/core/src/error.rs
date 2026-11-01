//! Unified error types for the VanGoghCat ecosystem.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VgcError {
    #[error("Invalid CatNodeID format: {0}")]
    InvalidNodeId(String),

    #[error("Invalid protocol version: expected {expected}, got {actual}")]
    InvalidProtocolVersion { expected: String, actual: String },

    #[error("Storage quota exceeded: requested {requested} bytes, but only {available} available")]
    StorageQuotaExceeded { requested: u64, available: u64 },

    #[error("Circle of peers is full (max 4 peers allowed)")]
    CircleFull,

    #[error("Cat Sanctity violation: {reason}")]
    CatSanctityViolation { reason: String },

    #[error("Cryptographic signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Invalid storage contract: {reason}")]
    InvalidContract { reason: String },

    #[error("Merkle proof verification failed at index {index}")]
    MerkleProofFailed { index: usize },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Nostr relay error: {0}")]
    NostrRelay(String),

    #[error("{0}")]
    Other(String),
}
