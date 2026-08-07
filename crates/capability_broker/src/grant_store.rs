use crate::token::{CapabilityType, GrantDecision};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

/// File-backed persistent store for user capability decisions.
#[derive(Debug, Clone)]
pub struct GrantStore {
    file_path: PathBuf,
    grants: HashMap<String, GrantDecision>, // key: "widget_id:capability"
}

impl GrantStore {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        let file_path = file_path.as_ref().to_path_buf();
        let grants = Self::load_from_disk(&file_path).unwrap_or_default();
        Self { file_path, grants }
    }

    fn key(widget_id: &str, capability: &CapabilityType) -> String {
        format!("{}:{}", widget_id, capability.as_str())
    }

    fn load_from_disk(path: &Path) -> Result<HashMap<String, GrantDecision>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read grant store file: {}", path.display()))?;
        let map: HashMap<String, GrantDecision> = serde_json::from_str(&content)?;
        Ok(map)
    }

    pub fn save_to_disk(&self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.grants)?;
        std::fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn record_decision(
        &mut self,
        widget_id: &str,
        capability: &CapabilityType,
        decision: GrantDecision,
    ) -> Result<()> {
        if decision == GrantDecision::AllowOnce {
            // AllowOnce is ephemeral, do not persist to disk
            return Ok(());
        }
        let key = Self::key(widget_id, capability);
        info!(widget_id = %widget_id, cap = %capability.as_str(), ?decision, "Persisting capability grant decision");
        self.grants.insert(key, decision);
        self.save_to_disk()
    }

    pub fn get_decision(&self, widget_id: &str, capability: &CapabilityType) -> Option<GrantDecision> {
        let key = Self::key(widget_id, capability);
        self.grants.get(&key).copied()
    }

    pub fn revoke_decision(&mut self, widget_id: &str, capability: &CapabilityType) -> Result<bool> {
        let key = Self::key(widget_id, capability);
        if self.grants.remove(&key).is_some() {
            self.save_to_disk()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_widget_grants(&self, widget_id: &str) -> HashMap<String, GrantDecision> {
        let prefix = format!("{}:", widget_id);
        self.grants
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, v)| (k.trim_start_matches(&prefix).to_string(), *v))
            .collect()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.grants.clear();
        self.save_to_disk()
    }
}
