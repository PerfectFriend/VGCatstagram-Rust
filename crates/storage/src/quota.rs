//! Atomic quota guard for the 8 GB storage limit.
//!
//! Enforces:
//! - Total storage limit: 8,000,000,000 bytes (8 GB)
//! - Own artworks reserve: 1,600,000,000 bytes (1.6 GB)
//! - Circle of 4 peers reserve: 6,400,000,000 bytes (6.4 GB, 1.6 GB each)
//!
//! Uses atomic checks and locks to guarantee NO race conditions when concurrent peers write.

use std::collections::HashMap;
use std::sync::RwLock;
use vangoghcat_core::{constants::*, VgcError};

#[derive(Debug, Default)]
pub struct QuotaGuard {
    inner: RwLock<QuotaState>,
}

#[derive(Debug, Default)]
struct QuotaState {
    pub own_used_bytes: u64,
    pub peer_used_bytes: HashMap<String, u64>,
}

impl QuotaGuard {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(QuotaState::default()),
        }
    }

    /// Attempts to allocate storage for own artwork.
    pub fn reserve_own(&self, bytes: u64) -> Result<(), VgcError> {
        let mut state = self
            .inner
            .write()
            .map_err(|_| VgcError::StorageQuotaExceeded {
                requested: bytes,
                available: 0,
            })?;

        let available = OWN_STORAGE_LIMIT_BYTES.saturating_sub(state.own_used_bytes);
        if bytes > available {
            return Err(VgcError::StorageQuotaExceeded {
                requested: bytes,
                available,
            });
        }

        state.own_used_bytes += bytes;
        Ok(())
    }

    /// Releases storage for own artwork.
    pub fn release_own(&self, bytes: u64) {
        if let Ok(mut state) = self.inner.write() {
            state.own_used_bytes = state.own_used_bytes.saturating_sub(bytes);
        }
    }

    /// Attempts to allocate storage for peer escrow.
    pub fn reserve_peer(&self, peer_id: &str, bytes: u64) -> Result<(), VgcError> {
        let mut state = self
            .inner
            .write()
            .map_err(|_| VgcError::StorageQuotaExceeded {
                requested: bytes,
                available: 0,
            })?;

        let current_peer_used = *state.peer_used_bytes.get(peer_id).unwrap_or(&0);
        let total_peer_used: u64 = state.peer_used_bytes.values().sum();

        // 1. Check peer individual limit
        let peer_available = PEER_STORAGE_LIMIT_BYTES.saturating_sub(current_peer_used);
        if bytes > peer_available {
            return Err(VgcError::StorageQuotaExceeded {
                requested: bytes,
                available: peer_available,
            });
        }

        // 2. Check total peer circle pool
        let pool_available = TOTAL_STORAGE_BYTES
            .saturating_sub(OWN_STORAGE_LIMIT_BYTES)
            .saturating_sub(total_peer_used);
        if bytes > pool_available {
            return Err(VgcError::StorageQuotaExceeded {
                requested: bytes,
                available: pool_available,
            });
        }

        *state
            .peer_used_bytes
            .entry(peer_id.to_string())
            .or_insert(0) += bytes;
        Ok(())
    }

    /// Releases storage for peer escrow.
    pub fn release_peer(&self, peer_id: &str, bytes: u64) {
        if let Ok(mut state) = self.inner.write() {
            if let Some(entry) = state.peer_used_bytes.get_mut(peer_id) {
                *entry = entry.saturating_sub(bytes);
            }
        }
    }

    /// Returns current usage statistics: (own_bytes, total_peer_bytes, total_used_bytes).
    pub fn stats(&self) -> (u64, u64, u64) {
        if let Ok(state) = self.inner.read() {
            let total_peer: u64 = state.peer_used_bytes.values().sum();
            (
                state.own_used_bytes,
                total_peer,
                state.own_used_bytes + total_peer,
            )
        } else {
            (0, 0, 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_guard_own_limit() {
        let guard = QuotaGuard::new();
        // Reserve 1 GB
        assert!(guard.reserve_own(1_000_000_000).is_ok());
        // Reserve another 700 MB -> should exceed 1.6 GB limit
        assert!(guard.reserve_own(700_000_000).is_err());
        // Release 500 MB
        guard.release_own(500_000_000);
        // Now 700 MB fits (1000 - 500 + 700 = 1200 MB <= 1600 MB)
        assert!(guard.reserve_own(700_000_000).is_ok());
    }

    #[test]
    fn test_quota_guard_peer_limit() {
        let guard = QuotaGuard::new();
        let peer = "catnode1peerAlice";
        assert!(guard.reserve_peer(peer, 1_500_000_000).is_ok());
        // Trying to exceed 1.6 GB for Alice
        assert!(guard.reserve_peer(peer, 200_000_000).is_err());
    }
}
