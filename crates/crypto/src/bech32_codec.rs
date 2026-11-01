//! Bech32 encoding and decoding for Nostr identities (npub, nsec) and CatNode identities.

use bech32::{primitives::decode::CheckedHrpstring, Bech32, Hrp};
use vangoghcat_core::VgcError;

/// Encodes raw bytes with specified HRP using Bech32.
pub fn encode_bech32(hrp_str: &str, data: &[u8]) -> Result<String, VgcError> {
    let hrp = Hrp::parse(hrp_str)
        .map_err(|e| VgcError::InvalidNodeId(format!("Invalid Bech32 HRP: {e}")))?;
    bech32::encode::<Bech32>(hrp, data)
        .map_err(|e| VgcError::InvalidNodeId(format!("Bech32 encode error: {e}")))
}

/// Decodes Bech32 string and returns (HRP, payload bytes).
pub fn decode_bech32(encoded: &str) -> Result<(String, Vec<u8>), VgcError> {
    let checked = CheckedHrpstring::new::<Bech32>(encoded)
        .map_err(|e| VgcError::InvalidNodeId(format!("Bech32 decode error: {e}")))?;
    let hrp = checked.hrp().to_string();
    let bytes: Vec<u8> = checked.byte_iter().collect();
    Ok((hrp, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bech32_roundtrip() {
        let hrp = "npub";
        let data = vec![1, 2, 3, 4, 5, 42, 99];
        let encoded = encode_bech32(hrp, &data).expect("should encode");
        assert!(encoded.starts_with("npub1"));

        let (decoded_hrp, decoded_data) = decode_bech32(&encoded).expect("should decode");
        assert_eq!(decoded_hrp, hrp);
        assert_eq!(decoded_data, data);
    }
}
