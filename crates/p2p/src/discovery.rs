//! # Zero-Configuration Local Swarm Discovery (UDP Beacon / Multicast)
//!
//! Provides automatic peer discovery across local area networks (Wi-Fi / Ethernet)
//! without reliance on external bootstrap nodes or central signaling servers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use vangoghcat_core::error::VgcError;

/// Default UDP port dedicated to sovereign VanGoghCat local discovery beacons.
pub const DEFAULT_DISCOVERY_PORT: u16 = 8769;

/// Beacon message broadcasted periodically by sovereign nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryBeacon {
    /// Sovereign CatNodeID of the broadcaster
    pub node_id: String,
    /// HTTP / WebSocket listening port of this node
    pub port: u16,
    /// Maximum capacity of this node's mutual storage circle (standard: 4 peers)
    pub circle_max_peers: usize,
    /// Current number of occupied slots in the circle
    pub circle_current_peers: usize,
    /// UNIX timestamp when the beacon was broadcast
    pub timestamp_secs: u64,
    /// Protocol version string
    pub version: String,
}

/// Metadata recorded for an active peer discovered over the local network.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub beacon: DiscoveryBeacon,
    pub remote_addr: SocketAddr,
    pub last_seen: Instant,
}

impl DiscoveredPeer {
    /// Checks whether this peer's mutual storage circle still has open slots.
    pub fn has_available_storage_slot(&self) -> bool {
        self.beacon.circle_current_peers < self.beacon.circle_max_peers
    }
}

/// Engine managing continuous background UDP beacon broadcast and peer tracking.
pub struct DiscoveryEngine {
    node_id: String,
    service_port: u16,
    discovery_port: u16,
    known_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl DiscoveryEngine {
    pub fn new(node_id: impl Into<String>, service_port: u16, discovery_port: u16) -> Self {
        Self {
            node_id: node_id.into(),
            service_port,
            discovery_port,
            known_peers: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Returns a list of currently active discovered peers.
    pub async fn active_peers(&self) -> Vec<DiscoveredPeer> {
        let guard = self.known_peers.read().await;
        guard.values().cloned().collect()
    }

    /// Returns peers that have open storage slots and are eligible for circle contracts.
    pub async fn candidate_peers_with_capacity(&self) -> Vec<DiscoveredPeer> {
        let guard = self.known_peers.read().await;
        guard
            .values()
            .filter(|p| p.has_available_storage_slot() && p.beacon.node_id != self.node_id)
            .cloned()
            .collect()
    }

    /// Prunes peers that have not transmitted a beacon within the specified timeout.
    pub async fn prune_inactive_peers(&self, timeout: Duration) -> usize {
        let mut guard = self.known_peers.write().await;
        let initial_len = guard.len();
        guard.retain(|_, peer| peer.last_seen.elapsed() < timeout);
        initial_len - guard.len()
    }

    /// Creates a beacon payload reflecting the current state.
    pub fn create_beacon(&self, circle_current_peers: usize) -> DiscoveryBeacon {
        let timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        DiscoveryBeacon {
            node_id: self.node_id.clone(),
            port: self.service_port,
            circle_max_peers: 4,
            circle_current_peers,
            timestamp_secs,
            version: "2.0.0".to_string(),
        }
    }

    /// Spawns the background discovery listener and announcer tasks.
    pub async fn start(
        &mut self,
        current_peers_fn: Arc<dyn Fn() -> usize + Send + Sync>,
    ) -> Result<(), VgcError> {
        let bind_addr = format!("0.0.0.0:{}", self.discovery_port);
        let socket = UdpSocket::bind(&bind_addr).await?;
        socket.set_broadcast(true)?;

        let socket = Arc::new(socket);
        let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        let known_peers = self.known_peers.clone();
        let my_node_id = self.node_id.clone();

        // 1. Inbound listener task
        let recv_socket = socket.clone();
        let mut shutdown_rx1 = shutdown_tx.subscribe();
        tokio::spawn(async move {
            let mut buf = [0u8; 2048];
            loop {
                tokio::select! {
                    _ = shutdown_rx1.recv() => {
                        debug!("Discovery inbound listener stopped.");
                        break;
                    }
                    result = recv_socket.recv_from(&mut buf) => {
                        match result {
                            Ok((size, remote_addr)) => {
                                if let Ok(beacon) = serde_json::from_slice::<DiscoveryBeacon>(&buf[..size]) {
                                    if beacon.node_id != my_node_id {
                                        let mut guard = known_peers.write().await;
                                        debug!("Discovered local peer: {} @ {}", beacon.node_id, remote_addr);
                                        guard.insert(
                                            beacon.node_id.clone(),
                                            DiscoveredPeer {
                                                beacon,
                                                remote_addr,
                                                last_seen: Instant::now(),
                                            },
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("UDP discovery recv error: {}", e);
                            }
                        }
                    }
                }
            }
        });

        // 2. Outbound periodic announcer task
        let send_socket = socket.clone();
        let mut shutdown_rx2 = shutdown_tx.subscribe();
        let broadcast_target = format!("255.255.255.255:{}", self.discovery_port);
        let my_id = self.node_id.clone();
        let service_port = self.service_port;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            loop {
                tokio::select! {
                    _ = shutdown_rx2.recv() => {
                        debug!("Discovery announcer stopped.");
                        break;
                    }
                    _ = interval.tick() => {
                        let current_count = (current_peers_fn)();
                        let timestamp_secs = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        let beacon = DiscoveryBeacon {
                            node_id: my_id.clone(),
                            port: service_port,
                            circle_max_peers: 4,
                            circle_current_peers: current_count,
                            timestamp_secs,
                            version: "2.0.0".to_string(),
                        };

                        if let Ok(bytes) = serde_json::to_vec(&beacon) {
                            let _ = send_socket.send_to(&bytes, &broadcast_target).await;
                        }
                    }
                }
            }
        });

        info!(
            "Zero-Config Local Swarm discovery active on UDP :{}",
            self.discovery_port
        );
        Ok(())
    }

    /// Stops background discovery tasks.
    pub fn stop(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_beacon_serialization() {
        let beacon = DiscoveryBeacon {
            node_id: "catnode1alice".to_string(),
            port: 8765,
            circle_max_peers: 4,
            circle_current_peers: 2,
            timestamp_secs: 1720000000,
            version: "2.0.0".to_string(),
        };

        let json = serde_json::to_string(&beacon).expect("serialize");
        let parsed: DiscoveryBeacon = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(beacon, parsed);
    }

    #[tokio::test]
    async fn test_prune_inactive_peers() {
        let engine = DiscoveryEngine::new("catnode1local", 8765, 9999);

        let peer = DiscoveredPeer {
            beacon: DiscoveryBeacon {
                node_id: "catnode1bob".to_string(),
                port: 8765,
                circle_max_peers: 4,
                circle_current_peers: 1,
                timestamp_secs: 1720000000,
                version: "2.0.0".to_string(),
            },
            remote_addr: "192.168.1.50:8765".parse().unwrap(),
            last_seen: Instant::now() - Duration::from_secs(15),
        };

        engine
            .known_peers
            .write()
            .await
            .insert(peer.beacon.node_id.clone(), peer);

        assert_eq!(engine.active_peers().await.len(), 1);
        let pruned = engine.prune_inactive_peers(Duration::from_secs(5)).await;
        assert_eq!(pruned, 1);
        assert_eq!(engine.active_peers().await.len(), 0);
    }
}
