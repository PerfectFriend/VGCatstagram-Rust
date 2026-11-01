//! Nostr Federation Protocol (NIP-01, NIP-05, NIP-57, NIP-94).
//!
//! Provides decentralized social broadcast of artworks, metadata federation,
//! and censorship-resistant cross-relay synchronization.

use serde::{Deserialize, Serialize};
use serde_json::json;
use vangoghcat_core::{types::NostrEvent, VgcError};
use vangoghcat_crypto::sha256_hex;

pub const NOSTR_KIND_SET_METADATA: u32 = 0;
pub const NOSTR_KIND_TEXT_NOTE: u32 = 1;
pub const NOSTR_KIND_FILE_METADATA: u32 = 1063; // NIP-94
pub const NOSTR_KIND_ZAP_REQUEST: u32 = 9734;   // NIP-57
pub const NOSTR_KIND_ZAP_RECEIPT: u32 = 9735;

pub const DEFAULT_NOSTR_RELAYS: &[&str] = &[
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.snort.social",
    "wss://nostr.wine",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrRelayStatus {
    pub url: String,
    pub is_connected: bool,
    pub published_events_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrArtworkMeta {
    pub artwork_id: String,
    pub title: String,
    pub image_url: String,
    pub content_sha256: String,
    pub dimensions: String,
    pub mime_type: String,
    pub author_npub: String,
}

pub struct NostrFederation;

impl NostrFederation {
    /// Serializes event into NIP-01 canonical JSON string:
    /// `[0, <pubkey>, <created_at>, <kind>, <tags>, <content>]`
    pub fn serialize_canonical(
        pubkey: &str,
        created_at: i64,
        kind: u32,
        tags: &[Vec<String>],
        content: &str,
    ) -> Result<String, VgcError> {
        let canonical_array = json!([0, pubkey, created_at, kind, tags, content]);
        serde_json::to_string(&canonical_array)
            .map_err(|e| VgcError::InvalidNodeId(format!("NIP-01 serialization error: {e}")))
    }

    /// Computes NIP-01 event ID as SHA-256 hex of canonical JSON serialization.
    pub fn compute_event_id(
        pubkey: &str,
        created_at: i64,
        kind: u32,
        tags: &[Vec<String>],
        content: &str,
    ) -> Result<String, VgcError> {
        let serialized = Self::serialize_canonical(pubkey, created_at, kind, tags, content)?;
        Ok(sha256_hex(serialized.as_bytes()))
    }

    /// Constructs a NIP-94 (kind 1063) artwork publication event.
    pub fn build_artwork_event(
        pubkey: &str,
        meta: &NostrArtworkMeta,
    ) -> Result<NostrEvent, VgcError> {
        let now = chrono::Utc::now().timestamp();
        let tags = vec![
            vec!["url".into(), meta.image_url.clone()],
            vec!["m".into(), meta.mime_type.clone()],
            vec!["x".into(), meta.content_sha256.clone()],
            vec!["dim".into(), meta.dimensions.clone()],
            vec!["title".into(), meta.title.clone()],
            vec!["t".into(), "vangoghcat".into()],
            vec!["t".into(), "cryptoart".into()],
            vec!["d".into(), meta.artwork_id.clone()],
        ];

        let content = format!(
            "Masterpiece «{}» minted on Van Gogh CryptoCats\nHash: {}",
            meta.title, meta.content_sha256
        );

        let id = Self::compute_event_id(pubkey, now, NOSTR_KIND_FILE_METADATA, &tags, &content)?;

        Ok(NostrEvent {
            id,
            pubkey: pubkey.to_string(),
            created_at: now,
            kind: NOSTR_KIND_FILE_METADATA,
            tags,
            content,
            sig: String::new(), // to be signed by keypair
        })
    }

    /// Validates integrity of a Nostr event against its ID.
    pub fn verify_event_integrity(event: &NostrEvent) -> bool {
        match Self::compute_event_id(
            &event.pubkey,
            event.created_at,
            event.kind,
            &event.tags,
            &event.content,
        ) {
            Ok(computed_id) => computed_id.eq_ignore_ascii_case(&event.id),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nip01_event_id_computation() {
        let pubkey = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        let created_at = 1700000000;
        let kind = 1;
        let tags: Vec<Vec<String>> = vec![vec!["p".into(), "someone".into()]];
        let content = "Hello Nostr from VanGoghCat!";

        let event_id = NostrFederation::compute_event_id(pubkey, created_at, kind, &tags, content)
            .expect("should compute");

        assert_eq!(event_id.len(), 64);
    }

    #[test]
    fn test_nip94_artwork_event_build_and_verify() {
        let pubkey = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        let meta = NostrArtworkMeta {
            artwork_id: "art_starry_cat".into(),
            title: "Starry Night Feline".into(),
            image_url: "https://vangoghcat.eth/art/1.png".into(),
            content_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            dimensions: "1024x1024".into(),
            mime_type: "image/png".into(),
            author_npub: "npub1xyz...".into(),
        };

        let event = NostrFederation::build_artwork_event(pubkey, &meta).expect("build event");
        assert_eq!(event.kind, NOSTR_KIND_FILE_METADATA);
        assert!(NostrFederation::verify_event_integrity(&event));
    }
}
