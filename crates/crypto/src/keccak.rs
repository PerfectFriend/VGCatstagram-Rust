//! Keccak-256 implementation for EVM-compatible commitments and sealed-bid auctions.

use tiny_keccak::{Hasher, Keccak};

/// Computes keccak256 hash (EVM native).
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}

/// Computes keccak256 commitment for sealed-bid auction: keccak256(bidder || amount_be || salt)
pub fn compute_bid_commitment(bidder: &str, amount: u64, salt: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(bidder.as_bytes());
    hasher.update(&amount.to_be_bytes());
    hasher.update(salt);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256_empty() {
        // keccak256("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let hash = keccak256(b"");
        assert_eq!(
            hex::encode(hash),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }
}
