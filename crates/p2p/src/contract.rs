//! Storage contract handshake and mutual circle of 4 peers protocol.

use serde::{Deserialize, Serialize};
use vangoghcat_core::{
    constants::*,
    types::{ContractStatus, StorageContractAck, StorageContractReq},
    VgcError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleOfPeers {
    pub peers: Vec<String>,
}

impl Default for CircleOfPeers {
    fn default() -> Self {
        Self { peers: Vec::new() }
    }
}

impl CircleOfPeers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn can_add_peer(&self) -> bool {
        self.peers.len() < MAX_PEER_CIRCLE_SIZE
    }

    pub fn add_peer(&mut self, peer_id: String) -> Result<(), VgcError> {
        if self.peers.contains(&peer_id) {
            return Ok(());
        }
        if self.peers.len() >= MAX_PEER_CIRCLE_SIZE {
            return Err(VgcError::CircleFull);
        }
        self.peers.push(peer_id);
        Ok(())
    }

    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.retain(|p| p != peer_id);
    }
}

pub fn handle_contract_request(
    req: &StorageContractReq,
    my_node_id: &str,
    circle: &mut CircleOfPeers,
) -> StorageContractAck {
    let contract_id = format!("ctr_{}_{}", my_node_id, req.sender_node_id);

    if !circle.can_add_peer() && !circle.peers.contains(&req.sender_node_id) {
        return StorageContractAck {
            msg_type: "CATP2P_STORAGE_CONTRACT_ACK".to_string(),
            version: "1.0".to_string(),
            contract_id,
            status: ContractStatus::QuotaFull,
            allocated_quota_bytes: 0,
            responder_node_id: my_node_id.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            signature: String::new(),
        };
    }

    let _ = circle.add_peer(req.sender_node_id.clone());

    StorageContractAck {
        msg_type: "CATP2P_STORAGE_CONTRACT_ACK".to_string(),
        version: "1.0".to_string(),
        contract_id,
        status: ContractStatus::Accepted,
        allocated_quota_bytes: req.allocated_quota_bytes.min(PEER_STORAGE_LIMIT_BYTES),
        responder_node_id: my_node_id.to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        signature: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_max_capacity() {
        let mut circle = CircleOfPeers::new();
        for i in 0..4 {
            assert!(circle.add_peer(format!("peer_{i}")).is_ok());
        }
        assert_eq!(circle.peers.len(), 4);
        assert!(!circle.can_add_peer());
        assert!(circle.add_peer("peer_5".into()).is_err());
    }
}
