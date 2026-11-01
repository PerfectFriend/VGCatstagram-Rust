//! AES-256-GCM authenticated encryption and decryption.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use vangoghcat_core::VgcError;

pub const NONCE_SIZE: usize = 12; // 96 bits

/// Encrypts plaintext using AES-256-GCM. Returns nonce prepended to ciphertext + tag.
pub fn encrypt_aes_gcm(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, VgcError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VgcError::InvalidNodeId(format!("Cipher init error: {e}")))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| VgcError::InvalidNodeId(format!("Encryption error: {e}")))?;

    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypts ciphertext where first 12 bytes are the nonce.
pub fn decrypt_aes_gcm(key: &[u8; 32], payload: &[u8]) -> Result<Vec<u8>, VgcError> {
    if payload.len() < NONCE_SIZE {
        return Err(VgcError::InvalidNodeId("Payload too short for nonce".into()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VgcError::InvalidNodeId(format!("Cipher init error: {e}")))?;

    let nonce = Nonce::from_slice(&payload[..NONCE_SIZE]);
    let ciphertext = &payload[NONCE_SIZE..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| VgcError::SignatureVerificationFailed(format!("AES-GCM decryption failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = [42u8; 32];
        let message = b"Secret Van Gogh Cat Private Key & Vault Payload";
        let encrypted = encrypt_aes_gcm(&key, message).expect("should encrypt");
        let decrypted = decrypt_aes_gcm(&key, &encrypted).expect("should decrypt");
        assert_eq!(decrypted, message);
    }
}
