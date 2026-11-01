//! System tray state, menus, and dynamic status badges for VanGoghCat desktop.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayMenuState {
    pub circle_status_label: String,
    pub storage_quota_label: String,
    pub is_online: bool,
    pub node_id: String,
}

impl TrayMenuState {
    pub fn new(node_id: &str, circle_count: usize, used_bytes: u64) -> Self {
        let circle_label = if circle_count >= 4 {
            "Circle: 5/5 Healthy (1 author + 4 custodians)".to_string()
        } else {
            format!("Circle: {}/5 Custodians Connected", circle_count + 1)
        };

        let used_mb = (used_bytes as f64) / 1_000_000.0;
        let quota_label = format!("Storage: {:.1} MB / 8,000 MB", used_mb);

        Self {
            circle_status_label: circle_label,
            storage_quota_label: quota_label,
            is_online: true,
            node_id: node_id.to_string(),
        }
    }

    /// Generates structured tray menu items list for UI renderers.
    pub fn menu_items(&self) -> Vec<TrayMenuItem> {
        vec![
            TrayMenuItem::Header(format!("Van Gogh CryptoCats — {}", self.node_id)),
            TrayMenuItem::Separator,
            TrayMenuItem::Status {
                label: self.circle_status_label.clone(),
                healthy: self.is_online,
            },
            TrayMenuItem::Status {
                label: self.storage_quota_label.clone(),
                healthy: true,
            },
            TrayMenuItem::Separator,
            TrayMenuItem::Action {
                id: "open_gallery".into(),
                label: "Open Sovereign Gallery".into(),
            },
            TrayMenuItem::Action {
                id: "copy_node_id".into(),
                label: "Copy Sovereign Node ID".into(),
            },
            TrayMenuItem::Separator,
            TrayMenuItem::Action {
                id: "quit".into(),
                label: "Quit Node".into(),
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrayMenuItem {
    Header(String),
    Status { label: String, healthy: bool },
    Action { id: String, label: String },
    Separator,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_state_healthy() {
        let tray = TrayMenuState::new("catnode1test", 4, 1_600_000_000);
        assert!(tray.circle_status_label.contains("5/5 Healthy"));
        assert!(tray.storage_quota_label.contains("1600.0 MB"));

        let items = tray.menu_items();
        assert_eq!(items.len(), 9);
    }
}
