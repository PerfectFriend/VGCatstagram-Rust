//! Simple Messaging Protocol (SMP) for anonymous asynchronous peer messaging.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;
use vangoghcat_core::VgcError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmpMessage {
    pub message_id: String,
    pub queue_id: String,
    pub sender_node_id: String,
    pub payload_encrypted_hex: String,
    pub timestamp: i64,
    pub ttl_seconds: u32,
}

pub struct SmpQueueManager {
    queues: RwLock<HashMap<String, Vec<SmpMessage>>>,
}

impl Default for SmpQueueManager {
    fn default() -> Self {
        Self {
            queues: RwLock::new(HashMap::new()),
        }
    }
}

impl SmpQueueManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a new random anonymous queue identifier: `smpq_<uuid>`.
    pub fn generate_queue_id() -> String {
        format!("smpq_{}", Uuid::new_v4().simple())
    }

    /// Enqueues an encrypted message into a target queue.
    pub fn push_message(
        &self,
        queue_id: &str,
        sender_node_id: &str,
        payload_encrypted_hex: &str,
        ttl_seconds: u32,
    ) -> Result<SmpMessage, VgcError> {
        let mut queues = self.queues.write().map_err(|_| {
            VgcError::InvalidNodeId("Failed to acquire lock for SMP queues".into())
        })?;

        let msg = SmpMessage {
            message_id: format!("msg_{}", Uuid::new_v4().simple()),
            queue_id: queue_id.to_string(),
            sender_node_id: sender_node_id.to_string(),
            payload_encrypted_hex: payload_encrypted_hex.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            ttl_seconds,
        };

        queues
            .entry(queue_id.to_string())
            .or_default()
            .push(msg.clone());

        Ok(msg)
    }

    /// Drains all available messages from a queue for the consumer.
    pub fn poll_queue(&self, queue_id: &str) -> Vec<SmpMessage> {
        if let Ok(mut queues) = self.queues.write() {
            queues.remove(queue_id).unwrap_or_default()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smp_queue_push_and_poll() {
        let smp = SmpQueueManager::new();
        let queue_id = SmpQueueManager::generate_queue_id();
        assert!(queue_id.starts_with("smpq_"));

        let sent = smp
            .push_message(&queue_id, "catnode1sender", "deadbeefcafe", 3600)
            .expect("should push");

        assert_eq!(sent.sender_node_id, "catnode1sender");

        let polled = smp.poll_queue(&queue_id);
        assert_eq!(polled.len(), 1);
        assert_eq!(polled[0].payload_encrypted_hex, "deadbeefcafe");

        // Subsequent poll is empty
        assert!(smp.poll_queue(&queue_id).is_empty());
    }
}
