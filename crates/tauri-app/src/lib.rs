//! Tauri v2 desktop IPC bridge and commands for VanGoghCat.
//!
pub mod tray;
pub use tray::*;

pub mod ipc {
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use vangoghcat_crypto::ZkpPromptEngine;
    use vangoghcat_p2p::daemon::{CatP2PDaemon, DaemonStatus};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppStateSummary {
        pub version: String,
        pub is_daemon_running: bool,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ZkpVerifyResult {
        pub is_valid: bool,
    }

    pub fn get_app_summary() -> AppStateSummary {
        AppStateSummary {
            version: env!("CARGO_PKG_VERSION").to_string(),
            is_daemon_running: true,
        }
    }

    pub async fn cmd_get_daemon_status(daemon: Arc<CatP2PDaemon>) -> DaemonStatus {
        daemon.get_status().await
    }

    pub fn cmd_verify_prompt(
        prompt: &str,
        salt_hex: &str,
        author_id: &str,
        expected_commitment_hex: &str,
    ) -> ZkpVerifyResult {
        let is_valid =
            ZkpPromptEngine::verify(prompt, salt_hex, author_id, expected_commitment_hex);
        ZkpVerifyResult { is_valid }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_app_summary() {
            let summary = get_app_summary();
            assert_eq!(summary.version, "2.0.0");
            assert!(summary.is_daemon_running);
        }

        #[test]
        fn test_ipc_verify_prompt() {
            let (commit, salt) = ZkpPromptEngine::commit("Vincent cat in Arles", "catnode1vincent");
            let salt_hex = hex::encode(salt);

            let res = cmd_verify_prompt(
                "Vincent cat in Arles",
                &salt_hex,
                "catnode1vincent",
                &commit.commitment_hex,
            );
            assert!(res.is_valid);
        }
    }
}
