use crate::transaction::ConfigTransaction;
use crate::types::{Snapshot, SnapshotMeta};
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

/// Manages desktop snapshot capture, persistence, restore, export, and import.
#[derive(Debug, Clone)]
pub struct SnapshotManager {
    storage_dir: PathBuf,
    max_snapshots: usize,
}

impl SnapshotManager {
    pub fn new<P: AsRef<Path>>(storage_dir: P, max_snapshots: usize) -> Self {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&storage_dir);
        Self {
            storage_dir,
            max_snapshots,
        }
    }

    /// Captures a desktop snapshot from given configuration payloads.
    pub fn create_snapshot(
        &self,
        name: &str,
        settings: serde_json::Value,
        layout: serde_json::Value,
        theme: serde_json::Value,
        now_ms: u64,
    ) -> Result<Snapshot> {
        let id = Uuid::new_v4().to_string();
        let snapshot = Snapshot {
            id: id.clone(),
            name: name.to_string(),
            created_at_ms: now_ms,
            aether_version: env!("CARGO_PKG_VERSION").to_string(),
            settings,
            layout,
            theme,
            widget_states: vec![],
            ai_layouts: vec![],
            plugins: vec![],
        };

        let file_path = self.snapshot_path(&id);
        let transaction = ConfigTransaction::new(&file_path);
        let payload = serde_json::to_value(&snapshot)?;
        transaction.write_atomic(&payload)?;

        info!(snapshot_id = %id, name = %name, "Desktop snapshot created");
        self.enforce_rotation()?;

        Ok(snapshot)
    }

    pub fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>> {
        if !self.storage_dir.exists() {
            return Ok(vec![]);
        }

        let mut list = vec![];
        for entry in std::fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(snapshot) = serde_json::from_str::<Snapshot>(&content) {
                        list.push(SnapshotMeta {
                            id: snapshot.id,
                            name: snapshot.name,
                            created_at_ms: snapshot.created_at_ms,
                            aether_version: snapshot.aether_version,
                        });
                    }
                }
            }
        }

        list.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
        Ok(list)
    }

    pub fn get_snapshot(&self, id: &str) -> Result<Snapshot> {
        let path = self.snapshot_path(id);
        if !path.exists() {
            return Err(anyhow!("Snapshot with ID '{}' does not exist", id));
        }
        let content = std::fs::read_to_string(&path)?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        Ok(snapshot)
    }

    pub fn delete_snapshot(&self, id: &str) -> Result<bool> {
        let path = self.snapshot_path(id);
        if path.exists() {
            std::fs::remove_file(&path)?;
            info!(snapshot_id = %id, "Deleted snapshot");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn export_snapshot<P: AsRef<Path>>(&self, id: &str, destination: P) -> Result<()> {
        let snapshot = self.get_snapshot(id)?;
        let payload = serde_json::to_value(&snapshot)?;
        let transaction = ConfigTransaction::new(destination);
        transaction.write_atomic(&payload)?;
        Ok(())
    }

    pub fn import_snapshot<P: AsRef<Path>>(&self, source_path: P) -> Result<Snapshot> {
        let content = std::fs::read_to_string(source_path.as_ref())
            .with_context(|| format!("Failed to read snapshot file: {}", source_path.as_ref().display()))?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        
        let target_path = self.snapshot_path(&snapshot.id);
        let transaction = ConfigTransaction::new(&target_path);
        let payload = serde_json::to_value(&snapshot)?;
        transaction.write_atomic(&payload)?;

        self.enforce_rotation()?;
        Ok(snapshot)
    }

    fn snapshot_path(&self, id: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.snapshot.json", id))
    }

    fn enforce_rotation(&self) -> Result<()> {
        let snapshots = self.list_snapshots()?;
        if snapshots.len() > self.max_snapshots {
            for old in &snapshots[self.max_snapshots..] {
                let _ = self.delete_snapshot(&old.id);
            }
        }
        Ok(())
    }
}
