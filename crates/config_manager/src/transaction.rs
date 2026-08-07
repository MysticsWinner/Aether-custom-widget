use crate::backup::ConfigBackupRotator;
use crate::validator::ConfigValidator;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

/// Manages atomic transactional writes to configuration files.
pub struct ConfigTransaction {
    target_path: PathBuf,
    backup_rotator: ConfigBackupRotator,
}

impl ConfigTransaction {
    pub fn new<P: AsRef<Path>>(target_path: P) -> Self {
        Self {
            target_path: target_path.as_ref().to_path_buf(),
            backup_rotator: ConfigBackupRotator::default(),
        }
    }

    pub fn with_backup_rotator(mut self, rotator: ConfigBackupRotator) -> Self {
        self.backup_rotator = rotator;
        self
    }

    /// Executes an atomic write of `payload` to `target_path`.
    pub fn write_atomic(&self, payload: &Value) -> Result<()> {
        ConfigValidator::validate_json_object(payload)?;

        let parent = self
            .target_path
            .parent()
            .context("Target path has no valid parent directory")?;
        std::fs::create_dir_all(parent)?;

        // 1. Write payload to unique temp file
        let temp_filename = format!(
            "{}.tmp.{}.json",
            self.target_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            Uuid::new_v4()
        );
        let temp_path = parent.join(temp_filename);

        let content = serde_json::to_string_pretty(payload)?;
        {
            let mut file = File::create(&temp_path)
                .with_context(|| format!("Failed to create temp config file: {}", temp_path.display()))?;
            file.write_all(content.as_bytes())?;
            file.sync_all()
                .with_context(|| format!("Failed to fsync temp file: {}", temp_path.display()))?;
        }

        // 2. Rotate previous generation backups
        if let Err(e) = self.backup_rotator.rotate_backups(&self.target_path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e).context("Failed to rotate configuration backups prior to atomic swap");
        }

        // 3. Atomic rename temp -> target_path
        if let Err(e) = std::fs::rename(&temp_path, &self.target_path) {
            let _ = std::fs::remove_file(&temp_path);
            // Attempt restore from backup on rename failure
            let _ = self.backup_rotator.restore_latest_backup(&self.target_path);
            return Err(e).context(format!(
                "Atomic rename failed from {} to {}",
                temp_path.display(),
                self.target_path.display()
            ));
        }

        info!(target = %self.target_path.display(), "Atomic config transaction committed successfully");
        Ok(())
    }
}
