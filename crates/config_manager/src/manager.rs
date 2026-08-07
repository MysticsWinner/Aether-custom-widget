use crate::migration::MigrationEngine;
use crate::snapshot::SnapshotManager;
use crate::transaction::ConfigTransaction;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// High-level façade orchestrating transactional writes, migrations, and desktop snapshots.
pub struct ConfigManager {
    base_dir: PathBuf,
    migration_engine: MigrationEngine,
    snapshot_manager: SnapshotManager,
}

impl ConfigManager {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();
        let snapshot_dir = base_dir.join("snapshots");
        let snapshot_manager = SnapshotManager::new(snapshot_dir, 20);

        Self {
            base_dir,
            migration_engine: MigrationEngine::new(),
            snapshot_manager,
        }
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn migration_engine_mut(&mut self) -> &mut MigrationEngine {
        &mut self.migration_engine
    }

    pub fn snapshot_manager(&self) -> &SnapshotManager {
        &self.snapshot_manager
    }

    /// Reads configuration file, applying migration engine if necessary.
    pub fn read_config<P: AsRef<Path>>(&self, rel_path: P, target_version: u32) -> Result<Value> {
        let path = self.base_dir.join(rel_path.as_ref());
        if !path.exists() {
            return Ok(serde_json::json!({
                "schema_version": target_version
            }));
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let mut value: Value = serde_json::from_str(&content)?;

        // Apply migrations
        let current_ver = self.migration_engine.migrate(&mut value, target_version)?;
        if current_ver != target_version {
            // Write upgraded file atomically
            self.write_config(rel_path, &value)?;
        }

        Ok(value)
    }

    /// Writes configuration file atomically via ConfigTransaction.
    pub fn write_config<P: AsRef<Path>>(&self, rel_path: P, value: &Value) -> Result<()> {
        let path = self.base_dir.join(rel_path.as_ref());
        let transaction = ConfigTransaction::new(path);
        transaction.write_atomic(value)?;
        Ok(())
    }
}
