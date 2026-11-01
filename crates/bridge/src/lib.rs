//! Censorship bridge module for Onion v3 and SOCKS5 failovers.

pub mod failover;
pub use failover::*;

pub mod bridge {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BridgeConfig {
        pub enabled: bool,
        pub socks5_proxy: Option<String>,
        pub tor_onion_endpoint: Option<String>,
    }

    impl BridgeConfig {
        pub fn with_socks5(proxy: &str) -> Self {
            Self {
                enabled: true,
                socks5_proxy: Some(proxy.to_string()),
                tor_onion_endpoint: None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_bridge_config_default() {
            let cfg = BridgeConfig::default();
            assert!(!cfg.enabled);
            assert!(cfg.socks5_proxy.is_none());
        }

        #[test]
        fn test_bridge_config_socks5() {
            let cfg = BridgeConfig::with_socks5("127.0.0.1:9050");
            assert!(cfg.enabled);
            assert_eq!(cfg.socks5_proxy.as_deref(), Some("127.0.0.1:9050"));
        }
    }
}
