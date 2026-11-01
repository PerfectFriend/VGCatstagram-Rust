//! Protocol constants ported from TypeScript `catP2PProtocol.ts`.

/// CatP2P protocol version.
pub const CATP2P_VERSION: &str = "1.0";

/// Total storage quota: 8 GB.
pub const TOTAL_STORAGE_BYTES: u64 = 8_000_000_000;

/// Own artworks reservation: 1.6 GB (~800 full-res artworks).
pub const OWN_STORAGE_BYTES: u64 = 1_600_000_000;
pub const OWN_STORAGE_LIMIT_BYTES: u64 = OWN_STORAGE_BYTES;

/// Storage reserved for mutual custodians: 6.4 GB (4 × 1.6 GB).
pub const FRIENDS_STORAGE_BYTES: u64 = 6_400_000_000;
pub const PEER_STORAGE_LIMIT_BYTES: u64 = 1_600_000_000;

/// Target replicas: 1 author + 4 custodians = 5 copies.
pub const TARGET_REPLICAS: u32 = 5;

/// Maximum custodians in the mutual guarantee circle.
pub const MAX_CUSTODIANS: u32 = 4;
pub const MAX_PEER_CIRCLE_SIZE: usize = 4;

/// Base Mainnet chain ID.
pub const BASE_CHAIN_ID_MAINNET: u64 = 8453;

/// Base Sepolia testnet chain ID.
pub const BASE_CHAIN_ID_SEPOLIA: u64 = 84532;

/// Maximum gasless (sponsored) transactions per smart account.
pub const MAX_SPONSORED_TRANSACTIONS: u32 = 10;

/// BIP-39 standard mnemonic word count.
pub const BIP39_WORD_COUNT: usize = 12;

/// CatP2P daemon default storage directory name.
pub const FELINE_STORAGE_DIR_NAME: &str = ".feline-p2p";

/// OPFS total quota (used in browser / OPFS storage engine).
pub const OPFS_TOTAL_QUOTA_BYTES: u64 = 8_000_000_000;
pub const OPFS_OWN_QUOTA_BYTES: u64 = 1_600_000_000;
pub const OPFS_PEERS_QUOTA_BYTES: u64 = 6_400_000_000;
pub const OPFS_ROOT_DIR: &str = "feline-p2p-cloud";
