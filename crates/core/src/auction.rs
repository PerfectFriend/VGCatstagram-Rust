//! Sealed-Bid confidential auction mechanics & ERC-4337 Account Abstraction structures.

use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuctionState {
    Active,
    Reveal,
    Settled,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuctionListing {
    pub auction_id: String,
    pub seller_address: String,
    pub nft_contract: String,
    pub token_id: u64,
    pub artwork_title: String,
    pub reserve_price_eth: String,
    pub commit_deadline: i64,
    pub reveal_deadline: i64,
    pub state: AuctionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_bid_eth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_bidder: Option<String>,
    pub commitments_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealedBidSecret {
    pub auction_id: String,
    pub bidder_address: String,
    pub actual_amount_wei: String,
    pub salt_hex: String,
    pub bid_hash_hex: String,
    pub committed_at: i64,
    pub revealed: bool,
}

/// ERC-4337 UserOperation for gasless sponsored transactions on Base L2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperation {
    pub sender: String,
    pub nonce: String,
    pub init_code: String,
    pub call_data: String,
    pub call_gas_limit: String,
    pub verification_gas_limit: String,
    pub pre_verification_gas: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub paymaster_and_data: String,
    pub signature: String,
}

pub struct AuctionEngine;

impl AuctionEngine {
    /// Computes EVM-compatible commitment hash: keccak256(amount_uint256 || salt_bytes32 || bidder_address)
    pub fn compute_bid_hash(
        amount_wei: u128,
        salt: &[u8; 32],
        bidder_address: &str,
    ) -> [u8; 32] {
        let mut hasher = Keccak::v256();

        // 1. Amount as 32-byte big-endian uint256
        let mut amount_bytes = [0u8; 32];
        amount_bytes[16..].copy_from_slice(&amount_wei.to_be_bytes());
        hasher.update(&amount_bytes);

        // 2. Salt 32 bytes
        hasher.update(salt);

        // 3. Bidder address (20 bytes normalized)
        let clean_addr = bidder_address.trim_start_matches("0x").trim_start_matches("0X");
        if let Ok(addr_bytes) = hex::decode(clean_addr) {
            let mut padded_addr = [0u8; 32];
            let offset = 32usize.saturating_sub(addr_bytes.len());
            padded_addr[offset..].copy_from_slice(&addr_bytes);
            hasher.update(&padded_addr);
        } else {
            hasher.update(bidder_address.as_bytes());
        }

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sealed_bid_hash_generation() {
        let salt = [0x77u8; 32];
        let bidder = "0x1234567890123456789012345678901234567890";
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH in wei

        let hash1 = AuctionEngine::compute_bid_hash(amount, &salt, bidder);
        let hash2 = AuctionEngine::compute_bid_hash(amount, &salt, bidder);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, [0u8; 32]);

        // Different amount gives different hash
        let hash3 = AuctionEngine::compute_bid_hash(amount + 1, &salt, bidder);
        assert_ne!(hash1, hash3);
    }
}
