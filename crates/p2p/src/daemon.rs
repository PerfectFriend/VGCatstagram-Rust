//! Core CatP2PDaemon orchestrator.

use crate::contract::CircleOfPeers;
use crate::smp::SmpQueueManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use vangoghcat_crypto::CatNodeIdentity;
use vangoghcat_storage::{QuotaGuard, StorageDb};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonStatus {
    pub node_id: String,
    pub circle_peers_count: usize,
    pub circle_is_full: bool,
    pub own_storage_bytes: u64,
    pub peer_storage_bytes: u64,
    pub total_storage_used_bytes: u64,
    pub is_online: bool,
}

pub struct CatP2PDaemon {
    pub identity: CatNodeIdentity,
    pub db: Arc<StorageDb>,
    pub quota: Arc<QuotaGuard>,
    pub circle: Arc<RwLock<CircleOfPeers>>,
    pub smp: Arc<SmpQueueManager>,
}

impl CatP2PDaemon {
    pub fn new(identity: CatNodeIdentity, db: Arc<StorageDb>) -> Self {
        Self {
            identity,
            db,
            quota: Arc::new(QuotaGuard::new()),
            circle: Arc::new(RwLock::new(CircleOfPeers::new())),
            smp: Arc::new(SmpQueueManager::new()),
        }
    }

    pub async fn get_status(&self) -> DaemonStatus {
        let circle = self.circle.read().await;
        let (own, peer, total) = self.quota.stats();

        DaemonStatus {
            node_id: self.identity.node_id.clone(),
            circle_peers_count: circle.peers.len(),
            circle_is_full: !circle.can_add_peer(),
            own_storage_bytes: own,
            peer_storage_bytes: peer,
            total_storage_used_bytes: total,
            is_online: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_status_lifecycle() {
        let identity = CatNodeIdentity::generate();
        let db = Arc::new(StorageDb::open_temporary().expect("db"));
        let daemon = CatP2PDaemon::new(identity, db);

        let status = daemon.get_status().await;
        assert!(status.node_id.starts_with("catnode1"));
        assert_eq!(status.circle_peers_count, 0);
        assert!(!status.circle_is_full);
        assert!(status.is_online);
    }
}
