//! Extended File Transfer Protocol (XFTP) for parallel chunk streaming.

use serde::{Deserialize, Serialize};
use vangoghcat_crypto::MerkleTree;

pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024; // 64 KB

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XftpFileChunk {
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub data: Vec<u8>,
    pub chunk_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XftpManifest {
    pub file_size: usize,
    pub chunk_size: usize,
    pub total_chunks: usize,
    pub merkle_root: String,
}

pub struct XftpEngine;

impl XftpEngine {
    /// Slices raw bytes into XFTP chunks and computes Merkle root.
    pub fn package(data: &[u8], chunk_size: usize) -> (XftpManifest, Vec<XftpFileChunk>) {
        let chunk_size = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
        let raw_chunks: Vec<Vec<u8>> = data
            .chunks(chunk_size)
            .map(|slice| slice.to_vec())
            .collect();

        let total_chunks = raw_chunks.len();
        let tree = MerkleTree::from_chunks(&raw_chunks);
        let merkle_root = hex::encode(tree.root);

        let file_chunks = raw_chunks
            .into_iter()
            .enumerate()
            .map(|(idx, chunk)| {
                let h = hex::encode(tree.leaves[idx]);
                XftpFileChunk {
                    chunk_index: idx,
                    total_chunks,
                    data: chunk,
                    chunk_hash: h,
                }
            })
            .collect();

        let manifest = XftpManifest {
            file_size: data.len(),
            chunk_size,
            total_chunks,
            merkle_root,
        };

        (manifest, file_chunks)
    }

    /// Reassembles chunks into original bytes.
    pub fn reassemble(mut chunks: Vec<XftpFileChunk>) -> Vec<u8> {
        chunks.sort_by_key(|c| c.chunk_index);
        let mut result = Vec::new();
        for c in chunks {
            result.extend(c.data);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xftp_package_and_reassemble() {
        let test_data = vec![0xAB; 200 * 1024]; // 200 KB
        let (manifest, chunks) = XftpEngine::package(&test_data, 64 * 1024);

        assert_eq!(manifest.total_chunks, 4);
        assert_eq!(chunks.len(), 4);
        assert!(!manifest.merkle_root.is_empty());

        let reassembled = XftpEngine::reassemble(chunks);
        assert_eq!(reassembled, test_data);
    }
}
