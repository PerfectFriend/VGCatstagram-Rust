//! Transport failover manager (Direct -> SOCKS5 -> Tor Onion).
//!
//! Provides automatic failover when connecting to peers across NATs, firewalls,
//! or censored networks.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportRoute {
    DirectWebRtc,
    Socks5Proxy,
    TorOnion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportFailoverConfig {
    pub allow_direct: bool,
    pub socks5_proxy: Option<String>,
    pub onion_proxy: Option<String>,
    pub max_retries_per_route: u32,
}

impl Default for TransportFailoverConfig {
    fn default() -> Self {
        Self {
            allow_direct: true,
            socks5_proxy: Some("127.0.0.1:9050".into()),
            onion_proxy: Some("127.0.0.1:9050".into()),
            max_retries_per_route: 3,
        }
    }
}

pub struct FailoverCoordinator {
    config: TransportFailoverConfig,
}

impl FailoverCoordinator {
    pub fn new(config: TransportFailoverConfig) -> Self {
        Self { config }
    }

    /// Selects available candidate routes in order of priority.
    pub fn route_candidates(&self) -> Vec<TransportRoute> {
        let mut routes = Vec::new();
        if self.config.allow_direct {
            routes.push(TransportRoute::DirectWebRtc);
        }
        if self.config.socks5_proxy.is_some() {
            routes.push(TransportRoute::Socks5Proxy);
        }
        if self.config.onion_proxy.is_some() {
            routes.push(TransportRoute::TorOnion);
        }
        routes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_candidates_order() {
        let coordinator = FailoverCoordinator::new(TransportFailoverConfig::default());
        let candidates = coordinator.route_candidates();
        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0], TransportRoute::DirectWebRtc);
        assert_eq!(candidates[1], TransportRoute::Socks5Proxy);
        assert_eq!(candidates[2], TransportRoute::TorOnion);
    }
}
