//! SHA-256 and content-addressed hashing utilities.

use sha2::{Digest, Sha256};

/// Computes SHA-256 hash of arbitrary byte slice, returning 32 bytes.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes SHA-256 and formats as lower-case hex string.
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256(data))
}

/// Computes double SHA-256 (hash256), common in Bitcoin/blockchain protocols.
pub fn hash256(data: &[u8]) -> [u8; 32] {
    sha256(&sha256(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(sha256_hex(b""), expected);
    }

    #[test]
    fn test_sha256_known_string() {
        let result = sha256_hex(b"VanGoghCat");
        assert_eq!(result.len(), 64);
    }
}
