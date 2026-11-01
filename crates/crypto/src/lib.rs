//! # VanGoghCat Crypto
//!
//! Cryptographic primitives for the Van Gogh CryptoCats ecosystem:
//! - SHA-256 hashing & content-addressed identifiers
//! - Base58 encoding (Bitcoin alphabet)
//! - Merkle tree construction & proof verification
//! - Ed25519 digital signatures (peer identity & Web of Trust)
//! - Bech32 encoding (Nostr npub/nsec)
//! - keccak256 for EVM-compatible commitment hashes
//! - AES-GCM 256-bit symmetric encryption

pub mod aes;
pub mod base58;
pub mod bech32_codec;
pub mod double_ratchet;
pub mod hash;
pub mod identity;
pub mod keccak;
pub mod merkle;
pub mod zkp;

pub use base58::encode_base58;
pub use bech32_codec::{decode_bech32, encode_bech32};
pub use double_ratchet::{RatchetHeader, RatchetMessage, RatchetSession};
pub use hash::*;
pub use identity::{generate_cat_node_id, CatNodeIdentity};
pub use keccak::keccak256;
pub use merkle::MerkleTree;
pub use zkp::{ZkpPromptCommitment, ZkpPromptEngine, ZkpPromptReveal};
