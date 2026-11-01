//! # Unified Configuration System
//!
//! Provides a single `VgcConfig` structure that can be loaded from:
//! 1. A TOML file (`vangoghcat.toml`)
//! 2. Environment variable overrides (`VGC_SERVER_PORT`, etc.)
//! 3. Compile-time defaults for every field
//!
//! All subsystems read their configuration from their respective sections
//! of this unified config rather than using scattered constants or env vars.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use crate::constants;

// ─────────────────────────────────────────────────────────────────────────────
// Top-Level Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Root configuration struct for the entire VanGoghCat system.
///
/// Example `vangoghcat.toml`:
/// ```toml
/// [server]
/// host = "0.0.0.0"
/// port = 8765
///
/// [storage]
/// base_dir = "~/.feline-p2p"
/// own_quota_bytes = 1600000000
///
/// [p2p]
/// max_custodians = 4
/// target_replicas = 5
///
/// [bridge]
/// enabled = false
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VgcConfig {
    /// Human-readable node name (displayed in tray, CLI banner, etc.)
    pub node_name: String,

    /// Server / HTTP settings
    pub server: ServerConfig,

    /// Persistent storage and quota enforcement
    pub storage: StorageConfig,

    /// Peer-to-peer networking and circle parameters
    pub p2p: P2pConfig,

    /// Cryptographic preferences
    pub crypto: CryptoConfig,

    /// Censorship-resistance transport bridge
    pub bridge: BridgeConfig,

    /// Logging and telemetry
    pub logging: LoggingConfig,
}

// ─────────────────────────────────────────────────────────────────────────────
// Section Configs
// ─────────────────────────────────────────────────────────────────────────────

/// Axum HTTP server and WebSocket relay settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Bind address for the Axum listener.
    pub host: IpAddr,

    /// Bind port for the Axum listener (REST + WebSocket).
    pub port: u16,

    /// Enable gzip/zstd response compression.
    pub enable_compression: bool,

    /// Maximum request body size in bytes (default: 10 MiB).
    pub max_body_bytes: usize,

    /// Rate-limit: max requests per second per IP.
    pub rate_limit_rps: u32,

    /// Rate-limit: burst tolerance (token bucket capacity).
    pub rate_limit_burst: u32,

    /// Origins allowed by CORS (empty = allow any).
    pub cors_allowed_origins: Vec<String>,
}

/// Persistent storage engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Root directory for the `.feline-p2p` storage tree.
    pub base_dir: PathBuf,

    /// Sled database subdirectory name.
    pub db_name: String,

    /// Own artworks quota in bytes.
    pub own_quota_bytes: u64,

    /// Total peer escrow quota in bytes (across all custodians).
    pub peer_total_quota_bytes: u64,

    /// Per-peer escrow slice in bytes.
    pub per_peer_quota_bytes: u64,

    /// Sled cache size in bytes (impacts RAM usage).
    pub sled_cache_bytes: u64,
}

/// Peer-to-peer circle parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct P2pConfig {
    /// Maximum custodians in the mutual guarantee circle.
    pub max_custodians: u32,

    /// Target replicas (1 author + N custodians).
    pub target_replicas: u32,

    /// SMP queue default TTL in seconds.
    pub smp_default_ttl_secs: u32,

    /// SMP maximum messages per queue.
    pub smp_max_queue_depth: usize,

    /// XFTP default chunk size in bytes.
    pub xftp_chunk_size: usize,

    /// Nostr relay URLs for NIP-01 / NIP-94 federation.
    pub nostr_relays: Vec<String>,
}

/// Cryptographic algorithm preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CryptoConfig {
    /// Whether to enable hardware-accelerated SHA-NI on x86_64.
    pub prefer_hw_sha: bool,

    /// Default AES-GCM nonce size in bytes (always 12 for AES-256-GCM).
    pub aes_nonce_bytes: usize,

    /// Double Ratchet max skip count before session reset.
    pub ratchet_max_skip: u32,
}

/// Transport bridge for censorship resistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BridgeConfig {
    /// Enable the censorship-resistance transport layer.
    pub enabled: bool,

    /// SOCKS5 proxy address (e.g., 127.0.0.1:9050).
    pub socks5_proxy: Option<String>,

    /// Tor onion v3 hidden service address.
    pub tor_onion_address: Option<String>,

    /// Failover order of transport strategies.
    pub failover_order: Vec<String>,
}

/// Logging and observability configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level filter (e.g., "info", "debug", "trace", "warn,hyper=info").
    pub level: String,

    /// Log output format: "full", "compact", "pretty", "json".
    pub format: String,

    /// Whether to include span events in logs.
    pub with_span_events: bool,

    /// Optional log file path (in addition to stderr).
    pub file: Option<PathBuf>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Default Implementations
// ─────────────────────────────────────────────────────────────────────────────

impl Default for VgcConfig {
    fn default() -> Self {
        Self {
            node_name: "VanGoghCat Node".to_string(),
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            p2p: P2pConfig::default(),
            crypto: CryptoConfig::default(),
            bridge: BridgeConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".parse().unwrap(),
            port: 8765,
            enable_compression: true,
            max_body_bytes: 10 * 1024 * 1024, // 10 MiB
            rate_limit_rps: 60,
            rate_limit_burst: 120,
            cors_allowed_origins: Vec::new(), // empty = allow any
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        let base_dir = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(constants::FELINE_STORAGE_DIR_NAME)
        } else {
            std::env::temp_dir().join(constants::FELINE_STORAGE_DIR_NAME)
        };

        Self {
            base_dir,
            db_name: "vgc_sled_db".to_string(),
            own_quota_bytes: constants::OWN_STORAGE_BYTES,
            peer_total_quota_bytes: constants::FRIENDS_STORAGE_BYTES,
            per_peer_quota_bytes: constants::PEER_STORAGE_LIMIT_BYTES,
            sled_cache_bytes: 64 * 1024 * 1024, // 64 MiB default sled cache
        }
    }
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            max_custodians: constants::MAX_CUSTODIANS,
            target_replicas: constants::TARGET_REPLICAS,
            smp_default_ttl_secs: 3600,
            smp_max_queue_depth: 1000,
            xftp_chunk_size: 64 * 1024, // 64 KiB
            nostr_relays: vec![
                "wss://relay.damus.io".to_string(),
                "wss://relay.snort.social".to_string(),
                "wss://nos.lol".to_string(),
            ],
        }
    }
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            prefer_hw_sha: true,
            aes_nonce_bytes: 12,
            ratchet_max_skip: 1000,
        }
    }
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            socks5_proxy: None,
            tor_onion_address: None,
            failover_order: vec![
                "direct".to_string(),
                "socks5".to_string(),
                "tor_onion".to_string(),
            ],
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "compact".to_string(),
            with_span_events: false,
            file: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Loading & Environment Overlay
// ─────────────────────────────────────────────────────────────────────────────

impl VgcConfig {
    /// Loads configuration from `vangoghcat.toml` at the given path,
    /// falling back to defaults for any missing fields, then applies
    /// environment variable overrides.
    pub fn load(toml_path: &Path) -> Result<Self, VgcConfigError> {
        let mut config = if toml_path.exists() {
            let raw = std::fs::read_to_string(toml_path).map_err(VgcConfigError::Io)?;
            toml::from_str(&raw).map_err(VgcConfigError::Parse)?
        } else {
            Self::default()
        };

        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    /// Applies environment variable overrides using `VGC_` prefix convention.
    ///
    /// Supported variables:
    /// - `VGC_NODE_NAME`
    /// - `VGC_SERVER_HOST`
    /// - `VGC_SERVER_PORT`
    /// - `VGC_STORAGE_DIR`
    /// - `VGC_LOG_LEVEL`
    /// - `VGC_BRIDGE_ENABLED`
    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("VGC_NODE_NAME") {
            self.node_name = v;
        }
        if let Ok(v) = std::env::var("VGC_SERVER_HOST") {
            if let Ok(addr) = v.parse() {
                self.server.host = addr;
            }
        }
        if let Ok(v) = std::env::var("VGC_SERVER_PORT") {
            if let Ok(port) = v.parse() {
                self.server.port = port;
            }
        }
        if let Ok(v) = std::env::var("PORT") {
            if let Ok(port) = v.parse() {
                self.server.port = port;
            }
        }
        if let Ok(v) = std::env::var("VGC_STORAGE_DIR") {
            self.storage.base_dir = PathBuf::from(v);
        }
        if let Ok(v) = std::env::var("VGC_LOG_LEVEL") {
            self.logging.level = v;
        }
        if let Ok(v) = std::env::var("VGC_BRIDGE_ENABLED") {
            self.bridge.enabled = v == "1" || v.eq_ignore_ascii_case("true");
        }
    }

    /// Validates the configuration for logical consistency.
    fn validate(&self) -> Result<(), VgcConfigError> {
        if self.server.port == 0 {
            return Err(VgcConfigError::Validation(
                "Server port cannot be 0".to_string(),
            ));
        }
        if self.storage.own_quota_bytes == 0 {
            return Err(VgcConfigError::Validation(
                "Own storage quota cannot be 0 bytes".to_string(),
            ));
        }
        if self.p2p.max_custodians == 0 {
            return Err(VgcConfigError::Validation(
                "Max custodians cannot be 0".to_string(),
            ));
        }
        Ok(())
    }

    /// Returns the full path to the Sled database directory.
    pub fn db_path(&self) -> PathBuf {
        self.storage.base_dir.join(&self.storage.db_name)
    }

    /// Returns the server bind address string (e.g., "0.0.0.0:8765").
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Configuration Errors
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum VgcConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Validation(String),
}

impl std::fmt::Display for VgcConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Config IO error: {}", e),
            Self::Parse(e) => write!(f, "Config parse error: {}", e),
            Self::Validation(msg) => write!(f, "Config validation error: {}", msg),
        }
    }
}

impl std::error::Error for VgcConfigError {}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let cfg = VgcConfig::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.server.port, 8765);
        assert_eq!(cfg.storage.own_quota_bytes, 1_600_000_000);
        assert_eq!(cfg.p2p.max_custodians, 4);
        assert!(!cfg.bridge.enabled);
    }

    #[test]
    fn test_bind_addr_formatting() {
        let cfg = VgcConfig::default();
        assert_eq!(cfg.bind_addr(), "0.0.0.0:8765");
    }

    #[test]
    fn test_db_path_composition() {
        let mut cfg = VgcConfig::default();
        cfg.storage.base_dir = PathBuf::from("/tmp/test-feline");
        cfg.storage.db_name = "my_db".to_string();
        assert_eq!(cfg.db_path(), PathBuf::from("/tmp/test-feline/my_db"));
    }

    #[test]
    fn test_validation_rejects_zero_port() {
        let mut cfg = VgcConfig::default();
        cfg.server.port = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_validation_rejects_zero_quota() {
        let mut cfg = VgcConfig::default();
        cfg.storage.own_quota_bytes = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_toml_roundtrip() {
        let cfg = VgcConfig::default();
        let toml_str = toml::to_string_pretty(&cfg).expect("serialize");
        let parsed: VgcConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(parsed.server.port, cfg.server.port);
        assert_eq!(parsed.storage.own_quota_bytes, cfg.storage.own_quota_bytes);
        assert_eq!(parsed.p2p.max_custodians, cfg.p2p.max_custodians);
    }

    #[test]
    fn test_partial_toml_fills_defaults() {
        let partial = r#"
            node_name = "My Art Node"
            [server]
            port = 9000
        "#;
        let cfg: VgcConfig = toml::from_str(partial).expect("parse partial");
        assert_eq!(cfg.node_name, "My Art Node");
        assert_eq!(cfg.server.port, 9000);
        // Defaults should fill in the rest
        assert_eq!(cfg.storage.own_quota_bytes, 1_600_000_000);
        assert_eq!(cfg.p2p.max_custodians, 4);
    }
}
