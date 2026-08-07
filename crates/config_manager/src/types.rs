use serde::{Deserialize, Serialize};

/// Versioned configuration header metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigHeader {
    pub schema_version: u32,
    #[serde(default)]
    pub last_modified_ms: u64,
}

/// Metadata for a saved widget inside a desktop snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetSnapshot {
    pub widget_id: String,
    pub version: String,
    pub config: serde_json::Value,
}

/// Metadata summary of a desktop snapshot for listing endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotMeta {
    pub id: String,
    pub name: String,
    pub created_at_ms: u64,
    pub aether_version: String,
}

/// Complete desktop snapshot holding all configuration files and subsystem state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub created_at_ms: u64,
    pub aether_version: String,
    pub settings: serde_json::Value,
    pub layout: serde_json::Value,
    pub theme: serde_json::Value,
    pub widget_states: Vec<WidgetSnapshot>,
    pub ai_layouts: Vec<serde_json::Value>,
    pub plugins: Vec<serde_json::Value>,
}
