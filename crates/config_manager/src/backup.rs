use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::info;

/// Handles rolling N-generation backups of configuration files.
#[derive(Debug, Clone)]
pub struct ConfigBackupRotator {
    max_generations: usize,
}

impl ConfigBackupRotator {
    pub fn new(max_generations: usize) -> Self {
        Self { max_generations }
    }

    /// Rotates backups for `target_file` before a write occurs.
    pub fn rotate_backups<P: AsRef<Path>>(&self, target_file: P) -> Result<()> {
        let target = target_file.as_ref();
        if !target.exists() {
            return Ok(());
        }

        // Shift oldest generations out: gen (max - 1) -> remove, gen (N) -> gen (N + 1)
        for gen in (1..self.max_generations).rev() {
            let src = self.backup_path_for(target, gen);
            let dst = self.backup_path_for(target, gen + 1);
            if src.exists() {
                if gen + 1 > self.max_generations {
                    let _ = std::fs::remove_file(&src);
                } else {
                    let _ = std::fs::rename(&src, &dst);
                }
            }
        }

        // Copy current file to gen 1 backup
        let gen1 = self.backup_path_for(target, 1);
        std::fs::copy(target, &gen1)?;
        info!(target = %target.display(), backup = %gen1.display(), "Created generation 1 backup");

        Ok(())
    }

    pub fn backup_path_for(&self, target: &Path, generation: usize) -> PathBuf {
        let file_name = target.file_name().unwrap_or_default().to_string_lossy();
        let backup_name = if generation == 1 {
            format!("{}.bak.json", file_name)
        } else {
            format!("{}.bak.{}.json", file_name, generation)
        };
        target.with_file_name(backup_name)
    }

    /// Restores the most recent backup generation for `target_file`.
    pub fn restore_latest_backup<P: AsRef<Path>>(&self, target_file: P) -> Result<bool> {
        let target = target_file.as_ref();
        for gen in 1..=self.max_generations {
            let backup = self.backup_path_for(target, gen);
            if backup.exists() {
                std::fs::copy(&backup, target)?;
                info!(target = %target.display(), backup = %backup.display(), "Restored target file from backup");
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Default for ConfigBackupRotator {
    fn default() -> Self {
        Self::new(5)
    }
}
