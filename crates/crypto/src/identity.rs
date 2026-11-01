//! Ed25519 peer identity and CatNodeID generation for the Van Gogh Cat P2P network.

use crate::base58::encode_base58;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use vangoghcat_core::VgcError;

pub const CAT_NODE_PREFIX: &str = "catnode1";

#[derive(Clone)]
pub struct CatNodeIdentity {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicPeerInfo {
    pub node_id: String,
    pub public_key_hex: String,
}

/// Generates a CatNodeID from an Ed25519 public key.
pub fn generate_cat_node_id(verifying_key: &VerifyingKey) -> String {
    let raw = verifying_key.as_bytes();
    let b58 = encode_base58(raw);
    format!("{}{}", CAT_NODE_PREFIX, &b58[..16.min(b58.len())])
}

impl CatNodeIdentity {
    /// Generates a new random Ed25519 identity.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let node_id = generate_cat_node_id(&verifying_key);

        Self {
            signing_key,
            verifying_key,
            node_id,
        }
    }

    /// Reconstructs identity from a 32-byte secret seed.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        let node_id = generate_cat_node_id(&verifying_key);

        Self {
            signing_key,
            verifying_key,
            node_id,
        }
    }

    /// Signs an arbitrary message using the node's private key.
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let sig: Signature = self.signing_key.sign(message);
        sig.to_vec()
    }

    /// Verifies a signature against this node's public key.
    pub fn verify(&self, message: &[u8], signature_bytes: &[u8]) -> Result<(), VgcError> {
        verify_signature(&self.verifying_key, message, signature_bytes)
    }

    pub fn public_info(&self) -> PublicPeerInfo {
        PublicPeerInfo {
            node_id: self.node_id.clone(),
            public_key_hex: hex::encode(self.verifying_key.as_bytes()),
        }
    }
}

pub fn verify_signature(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature_bytes: &[u8],
) -> Result<(), VgcError> {
    if signature_bytes.len() != 64 {
        return Err(VgcError::SignatureVerificationFailed(
            "Signature must be exactly 64 bytes".into(),
        ));
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(signature_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(message, &signature)
        .map_err(|e| VgcError::SignatureVerificationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation_and_signing() {
        let identity = CatNodeIdentity::generate();
        assert!(identity.node_id.starts_with(CAT_NODE_PREFIX));

        let msg = b"Van Gogh Cat Protocol Handshake v2";
        let sig = identity.sign(msg);
        assert!(identity.verify(msg, &sig).is_ok());

        // Tampered message must fail verification
        let tampered = b"Malicious Message";
        assert!(identity.verify(tampered, &sig).is_err());
    }
}
