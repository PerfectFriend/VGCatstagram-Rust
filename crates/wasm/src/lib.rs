//! # VanGoghCat WebAssembly (WASM) Engine
//!
//! Exposes cryptographic primitives, ZKP authorship commitments, and Merkle tree
//! proof verification to web browsers and React client runtimes at near-native speeds.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use vangoghcat_crypto::{
    decode_base58, decode_bech32, encode_base58, encode_bech32, sha256, MerkleProof, MerkleTree,
    ZkpPromptEngine,
};

#[derive(Serialize, Deserialize)]
pub struct WasmZkpCommitResult {
    pub commitment_hex: String,
    pub fingerprint_hex: String,
    pub salt_hex: String,
    pub author: String,
}

/// Generates a Zero-Knowledge authorship commitment for an AI prompt in the browser.
#[wasm_bindgen]
pub fn wasm_zkp_commit(prompt: &str, author: &str) -> Result<JsValue, JsValue> {
    let (commit, salt) = ZkpPromptEngine::commit(prompt, author);
    let result = WasmZkpCommitResult {
        commitment_hex: commit.commitment_hex,
        fingerprint_hex: commit.fingerprint_hex,
        salt_hex: hex::encode(salt),
        author: author.to_string(),
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Cryptographically verifies revealed prompt authorship against a public commitment.
#[wasm_bindgen]
pub fn wasm_zkp_verify(prompt: &str, salt_hex: &str, author: &str, commitment_hex: &str) -> bool {
    ZkpPromptEngine::verify(prompt, salt_hex, author, commitment_hex)
}

/// Computes the Merkle Root hash from an array of chunk SHA-256 hex strings.
#[wasm_bindgen]
pub fn wasm_merkle_root(chunks_hex: Vec<String>) -> Result<String, JsValue> {
    let mut leaves: Vec<[u8; 32]> = Vec::with_capacity(chunks_hex.len());
    for s in chunks_hex {
        let b =
            hex::decode(&s).map_err(|e| JsValue::from_str(&format!("Invalid chunk hex: {}", e)))?;
        if b.len() != 32 {
            return Err(JsValue::from_str(
                "Each chunk hash must be exactly 32 bytes",
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&b);
        leaves.push(arr);
    }

    let tree = MerkleTree::from_leaves(leaves);
    Ok(hex::encode(tree.root))
}

/// Verifies a Merkle proof against a root and leaf hash in WebAssembly.
#[wasm_bindgen]
pub fn wasm_verify_merkle_proof(
    root_hex: &str,
    leaf_hex: &str,
    index: usize,
    proof_siblings_hex: Vec<String>,
) -> Result<bool, JsValue> {
    let root_bytes = hex::decode(root_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid root hex: {}", e)))?;
    let leaf_bytes = hex::decode(leaf_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid leaf hex: {}", e)))?;

    if root_bytes.len() != 32 || leaf_bytes.len() != 32 {
        return Err(JsValue::from_str("Root and leaf must be 32 bytes"));
    }

    let mut root = [0u8; 32];
    root.copy_from_slice(&root_bytes);

    let mut leaf = [0u8; 32];
    leaf.copy_from_slice(&leaf_bytes);

    let mut siblings: Vec<[u8; 32]> = Vec::with_capacity(proof_siblings_hex.len());
    for s in proof_siblings_hex {
        let b = hex::decode(&s)
            .map_err(|e| JsValue::from_str(&format!("Invalid sibling hex: {}", e)))?;
        if b.len() != 32 {
            return Err(JsValue::from_str("Each sibling must be 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&b);
        siblings.push(arr);
    }

    let proof = MerkleProof {
        leaf_index: index,
        siblings,
    };

    Ok(MerkleTree::verify_proof(&root, &leaf, &proof))
}

/// Encodes raw bytes to sovereign Base58 string (Bitcoin/IPFS alphabet).
#[wasm_bindgen]
pub fn wasm_base58_encode(data: &[u8]) -> String {
    encode_base58(data)
}

/// Decodes sovereign Base58 string back to raw bytes.
#[wasm_bindgen]
pub fn wasm_base58_decode(encoded: &str) -> Result<Vec<u8>, JsValue> {
    decode_base58(encoded).map_err(|e| JsValue::from_str(&format!("Base58 decode error: {}", e)))
}

/// Encodes bytes to Nostr Bech32 string format (`npub`, `note`, `naddr`).
#[wasm_bindgen]
pub fn wasm_bech32_encode(hrp: &str, data: &[u8]) -> Result<String, JsValue> {
    encode_bech32(hrp, data).map_err(|e| JsValue::from_str(&format!("Bech32 encode error: {}", e)))
}

/// Decodes Nostr Bech32 string format.
#[wasm_bindgen]
pub fn wasm_bech32_decode(bech_str: &str) -> Result<Vec<u8>, JsValue> {
    let (_hrp, data) = decode_bech32(bech_str)
        .map_err(|e| JsValue::from_str(&format!("Bech32 decode error: {}", e)))?;
    Ok(data)
}

/// Computes SHA-256 hash string from string input.
#[wasm_bindgen]
pub fn wasm_sha256_hex(input: &str) -> String {
    let hash = sha256(input.as_bytes());
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_zkp_roundtrip() {
        let prompt = "Starry night feline portrait";
        let author = "catnode1alice";
        let (commit, salt) = ZkpPromptEngine::commit(prompt, author);
        let salt_hex = hex::encode(salt);

        let valid = wasm_zkp_verify(prompt, &salt_hex, author, &commit.commitment_hex);
        assert!(valid);

        let invalid = wasm_zkp_verify("Tampered prompt", &salt_hex, author, &commit.commitment_hex);
        assert!(!invalid);
    }

    #[test]
    fn test_wasm_base58_codecs() {
        let input = b"Van Gogh Sovereign Nodes 2026";
        let enc = wasm_base58_encode(input);
        let dec = wasm_base58_decode(&enc).expect("decode");
        assert_eq!(&dec[..], input);
    }

    #[test]
    fn test_wasm_merkle_root_computation() {
        let leaf1 = hex::encode(sha256(b"chunk1"));
        let leaf2 = hex::encode(sha256(b"chunk2"));
        let root = wasm_merkle_root(vec![leaf1, leaf2]).expect("merkle root");
        assert_eq!(root.len(), 64);
    }
}
