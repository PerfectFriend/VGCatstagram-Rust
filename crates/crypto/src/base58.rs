//! Base58 encoding and decoding using Bitcoin alphabet.

use bs58;
use vangoghcat_core::VgcError;

/// Encodes byte slice into Base58 string.
pub fn encode_base58(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// Decodes Base58 string into bytes.
pub fn decode_base58(encoded: &str) -> Result<Vec<u8>, VgcError> {
    bs58::decode(encoded)
        .into_vec()
        .map_err(|e| VgcError::InvalidNodeId(format!("Base58 decode error: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base58_roundtrip() {
        let original = b"Van Gogh Cat Protocol v2.0";
        let encoded = encode_base58(original);
        let decoded = decode_base58(&encoded).expect("should decode");
        assert_eq!(decoded, original);
    }
}
