//! Domain types ported from TypeScript `src/types.ts`.
//!
//! Every struct derives `Serialize` / `Deserialize` for JSON wire-compatibility
//! with the existing React/TypeScript frontend.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Artwork & Gallery
// ─────────────────────────────────────────────────────────────────────────────

/// Aspect ratio presets matching the frontend type `AspectRatioType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectRatio {
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "16:9")]
    Wide,
    #[serde(rename = "4:3")]
    Standard,
    #[serde(rename = "3:4")]
    Portrait,
    #[serde(rename = "9:16")]
    Tall,
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::Square
    }
}

/// A single color in the palette of an artwork.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSwatch {
    pub hex: String,
    pub name: String,
}

/// Origin source of an artwork.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtworkSource {
    Masterpiece,
    MyLocal,
    P2pFriend,
}

/// The central artwork entity — binary-compatible JSON with the TS `Artwork` interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_en: Option<String>,
    pub image_url: String,
    pub prompt: String,
    pub style: String,
    pub year: String,
    pub description: String,
    pub palette: Vec<ColorSwatch>,
    pub brushstroke_note: String,
    pub aspect_ratio: AspectRatio,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_user_created: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_used: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_svg_art: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub likes_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style_preset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_used: Option<String>,

    // ── P2P Fields ──────────────────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_peer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_backed_up_locally: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ArtworkSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_condemned: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condemned_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cat_profile_id: Option<String>,

    // ── ZKP Prompt Authorship ───────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zkp_commitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zkp_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zkp_revealed: Option<bool>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Digital Cat Passport & Identity
// ─────────────────────────────────────────────────────────────────────────────

/// Fur coat pattern variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoatPattern {
    SolidBlack,
    Tuxedo,
    GingerTabby,
    Calico,
    Siamese,
    White,
    GreySmoke,
    Other,
}

/// Digital cat passport with optional Soulbound Token (SBT) fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatProfile {
    pub id: String,
    pub name: String,
    pub breed: String,
    pub coat_pattern: CoatPattern,
    pub eye_color: String,
    pub distinctive_marks: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_photo_url: Option<String>,
    pub created_at: i64,
    pub artworks_count: u32,
    // SBT Fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbt_token_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbt_contract_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbt_tx_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbt_ipfs_cid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbt_sanctity_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbt_minted_at: Option<i64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// CatP2P Protocol Messages
// ─────────────────────────────────────────────────────────────────────────────

/// 8 GB Mutual Storage Node Quota.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageNodeQuota {
    /// 8,000,000,000 (8 GB)
    pub total_capacity_bytes: u64,
    /// 1,600,000,000 (1.6 GB)
    pub own_reserved_bytes: u64,
    /// 6,400,000,000 (6.4 GB, 4 × 1.6 GB)
    pub friends_reserved_bytes: u64,
    pub own_used_bytes: u64,
    pub friends_used_bytes: u64,
    /// Target: 4
    pub active_custodians_count: u32,
    pub whole_files_count: u32,
}

/// Mutual storage contract request (CATP2P_STORAGE_CONTRACT_REQ).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageContractReq {
    #[serde(rename = "type")]
    pub msg_type: String, // "CATP2P_STORAGE_CONTRACT_REQ"
    pub version: String,  // "1.0"
    pub sender_node_id: String,
    pub allocated_quota_bytes: u64,
    pub current_usage_bytes: u64,
    pub timestamp: i64,
    pub public_key: String,
    pub signature: String,
}

/// Mutual storage contract acknowledgement (CATP2P_STORAGE_CONTRACT_ACK).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageContractAck {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub version: String,
    pub contract_id: String,
    pub status: ContractStatus,
    pub allocated_quota_bytes: u64,
    pub responder_node_id: String,
    pub timestamp: i64,
    pub signature: String,
}

/// Status of a storage contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractStatus {
    Accepted,
    QuotaFull,
    Rejected,
}

/// File metadata inside a CatArtworkManifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkFileInfo {
    pub filename: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

/// Artist profile metadata inside a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestArtistProfile {
    pub author_id: String,
    pub author_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cat_name: Option<String>,
}

/// Metadata section of a CatArtworkManifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkMetadata {
    pub title: String,
    pub style: String,
    pub artist_profile: ManifestArtistProfile,
    pub description: String,
    pub color_palette: Vec<ColorSwatch>,
    pub model_engine: String,
    pub created_at: i64,
}

/// Replication target inside a CatArtworkManifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicationInfo {
    pub target_replicas: u32,
    pub custodian_peers: Vec<String>,
}

/// Content-addressed artwork manifest (CatP2P v1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatArtworkManifest {
    pub protocol: String, // "catp2p/1.0"
    pub artwork_id: String,
    pub file: ArtworkFileInfo,
    pub metadata: ArtworkMetadata,
    pub replication: ReplicationInfo,
    pub signature: String,
}

/// A custodian peer slot in the mutual guarantee circle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustodianSlot {
    pub index: u32,
    pub peer_id: String,
    pub peer_name: String,
    pub contract_id: String,
    pub status: CustodianStatus,
    pub quota_allocated_bytes: u64,
    pub quota_used_bytes: u64,
    pub last_ping_timestamp: i64,
    pub stored_artworks_count: u32,
}

/// Status of a custodian slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustodianStatus {
    Active,
    Pending,
    Vacant,
}

/// Full daemon status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonStatus {
    pub node_id: String,
    pub version: String,
    pub storage_dir: String,
    pub is_preallocated: bool,
    pub quota: StorageNodeQuota,
    pub active_contracts_count: u32,
    pub custodians: Vec<CustodianSlot>,
    pub uptime_seconds: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Nostr Federation Types (NIP-01, NIP-05, NIP-57, NIP-94)
// ─────────────────────────────────────────────────────────────────────────────

/// A Nostr keypair (hex-encoded private/public keys + bech32 npub/nsec).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrKeypair {
    pub private_key_hex: String,
    pub public_key_hex: String,
    pub npub: String,
    pub nsec: String,
}

/// Nostr event as per NIP-01.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostrEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: i64,
    pub kind: u32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Sealed-Bid Auction Types
// ─────────────────────────────────────────────────────────────────────────────

/// Phase of a sealed-bid auction lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuctionPhase {
    Active,
    Reveal,
    Settled,
}

/// Sealed-Bid Auction state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealedBidAuction {
    pub auction_id: String,
    pub artwork_id: String,
    pub creator: String,
    pub reserve_price_wei: String,
    pub phase: AuctionPhase,
    pub commit_deadline: i64,
    pub reveal_deadline: i64,
    pub bids_count: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// ERC-4337 Smart Account Types
// ─────────────────────────────────────────────────────────────────────────────

/// Smart account configuration for a specific chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartAccountConfig {
    pub chain_id: u64,
    pub entry_point_address: String,
    pub account_factory_address: String,
    pub bundler_url: String,
    pub paymaster_url: Option<String>,
}
