//! Zero-Knowledge Proof & cryptographic commitment for AI Prompt Authorship.
//!
//! Enables artists to prove they generated a specific artwork from a unique prompt
//! without revealing the prompt until they choose to publish it.

use crate::hash::sha256;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkpPromptCommitment {
    pub commitment_hex: String,
    pub fingerprint_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkpPromptReveal {
    pub prompt: String,
    pub salt_hex: String,
    pub author_id: String,
}

pub struct ZkpPromptEngine;

impl ZkpPromptEngine {
    /// Generates a cryptographic commitment for a prompt before generation.
    pub fn commit(prompt: &str, author_id: &str) -> (ZkpPromptCommitment, [u8; 32]) {
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let commitment = Self::compute_commitment(prompt, &salt, author_id);
        let fingerprint = sha256(prompt.as_bytes());

        (
            ZkpPromptCommitment {
                commitment_hex: hex::encode(commitment),
                fingerprint_hex: hex::encode(fingerprint),
            },
            salt,
        )
    }

    /// Computes commitment hash: sha256(sha256(prompt) || salt || author_id).
    pub fn compute_commitment(prompt: &str, salt: &[u8; 32], author_id: &str) -> [u8; 32] {
        let prompt_hash = sha256(prompt.as_bytes());
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&prompt_hash);
        buffer.extend_from_slice(salt);
        buffer.extend_from_slice(author_id.as_bytes());
        sha256(&buffer)
    }

    /// Verifies that the revealed prompt matches the committed hash.
    pub fn verify(
        prompt: &str,
        salt_hex: &str,
        author_id: &str,
        expected_commitment_hex: &str,
    ) -> bool {
        let salt_bytes = match hex::decode(salt_hex) {
            Ok(b) if b.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&b);
                arr
            }
            _ => return false,
        };

        let computed = Self::compute_commitment(prompt, &salt_bytes, author_id);
        hex::encode(computed).eq_ignore_ascii_case(expected_commitment_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zkp_prompt_commit_and_verify() {
        let prompt = "Majestic Van Gogh ginger cat sitting on a crescent moon";
        let author_id = "catnode1artistVincent";

        let (commitment, salt) = ZkpPromptEngine::commit(prompt, author_id);
        let salt_hex = hex::encode(salt);

        // Verification must succeed with correct parameters
        assert!(ZkpPromptEngine::verify(
            prompt,
            &salt_hex,
            author_id,
            &commitment.commitment_hex
        ));

        // Tampered prompt must fail
        assert!(!ZkpPromptEngine::verify(
            "Fake prompt",
            &salt_hex,
            author_id,
            &commitment.commitment_hex
        ));

        // Tampered author must fail
        assert!(!ZkpPromptEngine::verify(
            prompt,
            &salt_hex,
            "catnode1impostor",
            &commitment.commitment_hex
        ));
    }
}
