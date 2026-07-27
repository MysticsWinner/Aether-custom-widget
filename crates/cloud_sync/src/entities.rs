use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 1. Layout Entity: Screen bounds, flexbox dimensions, and multi-monitor positioning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutEntity {
    pub layout_id: String,
    pub display_id: String,
    pub bounds_x: f32,
    pub bounds_y: f32,
    pub width: f32,
    pub height: f32,
}

/// 2. Theme Entity: Theme schema ID, color palette tokens, and font mappings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeEntity {
    pub theme_id: String,
    pub color_tokens: HashMap<String, String>,
    pub font_family: String,
}

/// 3. Settings Entity: Host and widget configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingsEntity {
    pub setting_id: String,
    pub key: String,
    pub value_json: String,
}

/// 4. Plugin Entity: Sandboxed plugin manifest pins, version requirements, enabled state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginEntity {
    pub plugin_id: String,
    pub pinned_version: String,
    pub enabled: bool,
}

/// 5. Device Entity: Workstation hardware profiles and display monitor configs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceEntity {
    pub device_id: String,
    pub hostname: String,
    pub monitor_count: u32,
    pub os_build: String,
}

/// 6. Account Entity: User authentication credentials and encrypted session tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountEntity {
    pub user_id: String,
    pub email: String,
    pub auth_token_encrypted: String,
}

/// Unified Cloud Synchronized Entity Enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncEntity {
    Layout(LayoutEntity),
    Theme(ThemeEntity),
    Settings(SettingsEntity),
    Plugin(PluginEntity),
    Device(DeviceEntity),
    Account(AccountEntity),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_entities_serialization() {
        let layout = SyncEntity::Layout(LayoutEntity {
            layout_id: "main_desktop_v1".into(),
            display_id: "MONITOR_1".into(),
            bounds_x: 100.0,
            bounds_y: 200.0,
            width: 400.0,
            height: 300.0,
        });

        let json = serde_json::to_string(&layout).unwrap();
        let parsed: SyncEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layout);
    }
}
