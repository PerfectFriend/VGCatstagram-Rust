//! Tauri v2 desktop IPC bridge and commands for VanGoghCat.

pub mod ipc {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct AppStateSummary {
        pub version: String,
        pub is_daemon_running: bool,
    }

    pub fn get_app_summary() -> AppStateSummary {
        AppStateSummary {
            version: env!("CARGO_PKG_VERSION").to_string(),
            is_daemon_running: true,
        }
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
    }
}
