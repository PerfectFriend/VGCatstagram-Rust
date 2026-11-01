//! Merkle tree construction and proof verification for XFTP swarms.

use crate::hash::sha256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub siblings: Vec<[u8; 32]>,
}

#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub leaves: Vec<[u8; 32]>,
    pub root: [u8; 32],
}

impl MerkleTree {
    /// Builds a Merkle tree from raw leaf chunks.
    pub fn from_chunks(chunks: &[Vec<u8>]) -> Self {
        let leaves: Vec<[u8; 32]> = chunks.iter().map(|c| sha256(c)).collect();
        Self::from_leaves(leaves)
    }

    /// Builds a Merkle tree from already-hashed leaves.
    pub fn from_leaves(leaves: Vec<[u8; 32]>) -> Self {
        if leaves.is_empty() {
            return Self {
                leaves: vec![],
                root: [0u8; 32],
            };
        }

        let mut current_level = leaves.clone();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                if i + 1 < current_level.len() {
                    let mut combined = [0u8; 64];
                    combined[..32].copy_from_slice(&current_level[i]);
                    combined[32..].copy_from_slice(&current_level[i + 1]);
                    next_level.push(sha256(&combined));
                } else {
                    // Duplicate lone leaf to balance tree
                    let mut combined = [0u8; 64];
                    combined[..32].copy_from_slice(&current_level[i]);
                    combined[32..].copy_from_slice(&current_level[i]);
                    next_level.push(sha256(&combined));
                }
            }
            current_level = next_level;
        }

        let root = current_level[0];
        Self { leaves, root }
    }

    /// Generates audit proof for a given leaf index.
    pub fn generate_proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut siblings = Vec::new();
        let mut current_level = self.leaves.clone();
        let mut idx = index;

        while current_level.len() > 1 {
            let sibling_idx = if idx % 2 == 0 {
                if idx + 1 < current_level.len() {
                    idx + 1
                } else {
                    idx // duplicated leaf
                }
            } else {
                idx - 1
            };
            siblings.push(current_level[sibling_idx]);

            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                if i + 1 < current_level.len() {
                    let mut combined = [0u8; 64];
                    combined[..32].copy_from_slice(&current_level[i]);
                    combined[32..].copy_from_slice(&current_level[i + 1]);
                    next_level.push(sha256(&combined));
                } else {
                    let mut combined = [0u8; 64];
                    combined[..32].copy_from_slice(&current_level[i]);
                    combined[32..].copy_from_slice(&current_level[i]);
                    next_level.push(sha256(&combined));
                }
            }
            idx /= 2;
            current_level = next_level;
        }

        Some(MerkleProof {
            leaf_index: index,
            siblings,
        })
    }

    /// Verifies that a given leaf with proof matches the expected Merkle root.
    pub fn verify_proof(root: &[u8; 32], leaf: &[u8; 32], proof: &MerkleProof) -> bool {
        let mut current = *leaf;
        let mut idx = proof.leaf_index;

        for sibling in &proof.siblings {
            let mut combined = [0u8; 64];
            if idx % 2 == 0 {
                combined[..32].copy_from_slice(&current);
                combined[32..].copy_from_slice(sibling);
            } else {
                combined[..32].copy_from_slice(sibling);
                combined[32..].copy_from_slice(&current);
            }
            current = sha256(&combined);
            idx /= 2;
        }

        &current == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_proof_verification() {
        let chunks: Vec<Vec<u8>> = (0..8).map(|i| vec![i as u8; 1024]).collect();
        let tree = MerkleTree::from_chunks(&chunks);

        for i in 0..chunks.len() {
            let proof = tree.generate_proof(i).expect("proof exists");
            let leaf = tree.leaves[i];
            assert!(MerkleTree::verify_proof(&tree.root, &leaf, &proof));
        }
    }
}
