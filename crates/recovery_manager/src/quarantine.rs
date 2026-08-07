use crate::types::QuarantineRecord;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

/// File-backed persistent store for quarantined widgets.
#[derive(Debug, Clone)]
pub struct QuarantineStore {
    file_path: PathBuf,
    quarantined: HashMap<String, QuarantineRecord>,
}

impl QuarantineStore {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        let file_path = file_path.as_ref().to_path_buf();
        let quarantined = Self::load_from_disk(&file_path).unwrap_or_default();
        Self {
            file_path,
            quarantined,
        }
    }

    fn load_from_disk(path: &Path) -> Result<HashMap<String, QuarantineRecord>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read quarantine file: {}", path.display()))?;
        let records: Vec<QuarantineRecord> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse quarantine store JSON")?;
        let map = records.into_iter().map(|r| (r.widget_id.clone(), r)).collect();
        Ok(map)
    }

    pub fn save_to_disk(&self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let records: Vec<&QuarantineRecord> = self.quarantined.values().collect();
        let content = serde_json::to_string_pretty(&records)?;
        std::fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn quarantine(&mut self, record: QuarantineRecord) -> Result<()> {
        info!(widget_id = %record.widget_id, reason = %record.reason, "Quarantining widget");
        self.quarantined.insert(record.widget_id.clone(), record);
        self.save_to_disk()
    }

    pub fn remove(&mut self, widget_id: &str) -> Result<bool> {
        if self.quarantined.remove(widget_id).is_some() {
            self.save_to_disk()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn is_quarantined(&self, widget_id: &str) -> bool {
        self.quarantined.contains_key(widget_id)
    }

    pub fn list(&self) -> Vec<QuarantineRecord> {
        self.quarantined.values().cloned().collect()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.quarantined.clear();
        self.save_to_disk()
    }
}
