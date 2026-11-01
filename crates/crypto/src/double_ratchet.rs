//! Signal-style Double Ratchet protocol for «Ван Гог Whispers» E2EE chat.
//!
//! Provides Perfect Forward Secrecy (PFS) and Break-in Recovery for peer messages:
//! 1. Diffie-Hellman Ratchet (X25519)
//! 2. Symmetric KDF Chain Ratchet (HKDF-SHA256)
//! 3. AES-256-GCM authenticated message encryption

use crate::aes::{decrypt_aes_gcm, encrypt_aes_gcm};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use vangoghcat_core::VgcError;
use x25519_dalek::{PublicKey, StaticSecret};

const HKDF_INFO_ROOT: &[u8] = b"VanGoghWhispers::DoubleRatchet::RootKey::v1";
const HKDF_INFO_CHAIN: &[u8] = b"VanGoghWhispers::DoubleRatchet::ChainKey::v1";
const HKDF_INFO_MSG: &[u8] = b"VanGoghWhispers::DoubleRatchet::MessageKey::v1";
const HKDF_SALT: &[u8] = b"KotikiCryptoCats_Salt_2026";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetHeader {
    pub dh_pub_hex: String,
    pub prev_chain_len: u32,
    pub msg_num: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetMessage {
    pub header: RatchetHeader,
    pub ciphertext_hex: String,
}

pub struct RatchetSession {
    pub dh_secret: StaticSecret,
    pub dh_public: PublicKey,
    pub their_dh_public: Option<PublicKey>,
    pub root_key: [u8; 32],
    pub send_chain_key: Option<[u8; 32]>,
    pub recv_chain_key: Option<[u8; 32]>,
    pub send_msg_num: u32,
    pub recv_msg_num: u32,
    pub prev_chain_len: u32,
}

impl RatchetSession {
    /// Initializes an initiator session (Alice) with Bob's public prekey and initial shared secret.
    pub fn init_alice(shared_master_secret: &[u8; 32], bob_dh_public: PublicKey) -> Self {
        let dh_secret = StaticSecret::random_from_rng(OsRng);
        let dh_public = PublicKey::from(&dh_secret);

        // DH ratchet step
        let dh_output = dh_secret.diffie_hellman(&bob_dh_public);
        let (root_key, send_chain_key) = kdf_rk(shared_master_secret, dh_output.as_bytes());

        Self {
            dh_secret,
            dh_public,
            their_dh_public: Some(bob_dh_public),
            root_key,
            send_chain_key: Some(send_chain_key),
            recv_chain_key: None,
            send_msg_num: 0,
            recv_msg_num: 0,
            prev_chain_len: 0,
        }
    }

    /// Initializes a recipient session (Bob) with Alice's identity and initial shared secret.
    pub fn init_bob(
        shared_master_secret: &[u8; 32],
        bob_keypair: (StaticSecret, PublicKey),
    ) -> Self {
        Self {
            dh_secret: bob_keypair.0,
            dh_public: bob_keypair.1,
            their_dh_public: None,
            root_key: *shared_master_secret,
            send_chain_key: None,
            recv_chain_key: None,
            send_msg_num: 0,
            recv_msg_num: 0,
            prev_chain_len: 0,
        }
    }

    /// Encrypts plaintext message, advancing the sending chain.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<RatchetMessage, VgcError> {
        let chain_key = self
            .send_chain_key
            .ok_or_else(|| VgcError::InvalidNodeId("Send chain key not initialized".into()))?;

        let (next_chain_key, message_key) = kdf_ck(&chain_key);
        self.send_chain_key = Some(next_chain_key);

        let header = RatchetHeader {
            dh_pub_hex: hex::encode(self.dh_public.as_bytes()),
            prev_chain_len: self.prev_chain_len,
            msg_num: self.send_msg_num,
        };

        self.send_msg_num += 1;

        let ciphertext = encrypt_aes_gcm(&message_key, plaintext)?;

        Ok(RatchetMessage {
            header,
            ciphertext_hex: hex::encode(ciphertext),
        })
    }

    /// Decrypts message, ratcheting DH and receiving chain as needed.
    pub fn decrypt(&mut self, msg: &RatchetMessage) -> Result<Vec<u8>, VgcError> {
        let their_dh_bytes = hex::decode(&msg.header.dh_pub_hex)
            .map_err(|e| VgcError::InvalidNodeId(format!("Hex decode error: {e}")))?;
        if their_dh_bytes.len() != 32 {
            return Err(VgcError::InvalidNodeId("Invalid public key length".into()));
        }

        let mut pub_arr = [0u8; 32];
        pub_arr.copy_from_slice(&their_dh_bytes);
        let their_dh = PublicKey::from(pub_arr);

        // Check if ratchet turn has occurred (new DH key received)
        let is_new_dh = match &self.their_dh_public {
            Some(current) => current != &their_dh,
            None => true,
        };

        if is_new_dh {
            self.prev_chain_len = self.send_msg_num;
            self.send_msg_num = 0;
            self.recv_msg_num = 0;
            self.their_dh_public = Some(their_dh);

            // 1. Ratchet Recv Chain
            let dh_recv = self.dh_secret.diffie_hellman(&their_dh);
            let (new_root, recv_ck) = kdf_rk(&self.root_key, dh_recv.as_bytes());
            self.root_key = new_root;
            self.recv_chain_key = Some(recv_ck);

            // 2. Ratchet Send Chain with a fresh keypair
            self.dh_secret = StaticSecret::random_from_rng(OsRng);
            self.dh_public = PublicKey::from(&self.dh_secret);

            let dh_send = self.dh_secret.diffie_hellman(&their_dh);
            let (next_root, send_ck) = kdf_rk(&self.root_key, dh_send.as_bytes());
            self.root_key = next_root;
            self.send_chain_key = Some(send_ck);
        }

        let recv_ck = self
            .recv_chain_key
            .ok_or_else(|| VgcError::InvalidNodeId("Recv chain not available".into()))?;

        let (next_recv_ck, message_key) = kdf_ck(&recv_ck);
        self.recv_chain_key = Some(next_recv_ck);
        self.recv_msg_num += 1;

        let cipher_bytes = hex::decode(&msg.ciphertext_hex)
            .map_err(|e| VgcError::InvalidNodeId(format!("Ciphertext hex error: {e}")))?;

        decrypt_aes_gcm(&message_key, &cipher_bytes)
    }
}

/// Root KDF deriving (Next Root Key, Chain Key) from current root and DH output.
fn kdf_rk(root_key: &[u8; 32], dh_output: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::new(Some(root_key), dh_output);
    let mut okm = [0u8; 64];
    hk.expand(HKDF_INFO_ROOT, &mut okm)
        .expect("64 bytes valid for HKDF-SHA256");

    let mut next_rk = [0u8; 32];
    let mut chain_key = [0u8; 32];
    next_rk.copy_from_slice(&okm[..32]);
    chain_key.copy_from_slice(&okm[32..]);
    (next_rk, chain_key)
}

/// Chain KDF deriving (Next Chain Key, Message Key) from current chain key.
fn kdf_ck(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hk_chain = Hkdf::<Sha256>::new(Some(HKDF_SALT), chain_key);
    let mut next_ck = [0u8; 32];
    hk_chain
        .expand(HKDF_INFO_CHAIN, &mut next_ck)
        .expect("32 bytes valid");

    let mut msg_key = [0u8; 32];
    hk_chain
        .expand(HKDF_INFO_MSG, &mut msg_key)
        .expect("32 bytes valid");

    (next_ck, msg_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_ratchet_conversation() {
        let master_secret = [0x5Au8; 32];

        // Bob prepares keypair
        let bob_secret = StaticSecret::random_from_rng(OsRng);
        let bob_public = PublicKey::from(&bob_secret);

        // Alice initializes session with Bob's public key
        let mut alice = RatchetSession::init_alice(&master_secret, bob_public);
        let mut bob = RatchetSession::init_bob(&master_secret, (bob_secret, bob_public));

        // Alice sends message 1
        let msg1 = alice.encrypt(b"Hello Bob from Alice!").expect("encrypt");
        let dec1 = bob.decrypt(&msg1).expect("decrypt");
        assert_eq!(dec1, b"Hello Bob from Alice!");

        // Bob responds to Alice
        let msg2 = bob
            .encrypt(b"Hey Alice, secret received!")
            .expect("encrypt");
        let dec2 = alice.decrypt(&msg2).expect("decrypt");
        assert_eq!(dec2, b"Hey Alice, secret received!");

        // Alice replies back
        let msg3 = alice
            .encrypt(b"Perfect Forward Secrecy guaranteed!")
            .expect("encrypt");
        let dec3 = bob.decrypt(&msg3).expect("decrypt");
        assert_eq!(dec3, b"Perfect Forward Secrecy guaranteed!");
    }
}
