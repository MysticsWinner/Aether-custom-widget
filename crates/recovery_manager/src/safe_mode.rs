use crate::types::LaunchMode;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Guard responsible for Safe Mode sentinel management and crash count evaluation.
#[derive(Debug, Clone)]
pub struct SafeModeGuard {
    sentinel_path: PathBuf,
    counter_path: PathBuf,
    max_engine_crashes: u32,
}

impl SafeModeGuard {
    pub fn new<P: AsRef<Path>>(base_dir: P, max_engine_crashes: u32) -> Self {
        let base_dir = base_dir.as_ref();
        Self {
            sentinel_path: base_dir.join(".safe_mode_sentinel"),
            counter_path: base_dir.join(".safe_mode_counter"),
            max_engine_crashes,
        }
    }

    /// Evaluates current launch mode based on sentinel & crash counter existence.
    pub fn evaluate_and_arm(&self) -> Result<LaunchMode> {
        let mut crash_count = self.read_crash_counter();

        // If sentinel file exists from previous run, it implies ungraceful shutdown/crash.
        if self.sentinel_path.exists() {
            crash_count += 1;
            self.write_crash_counter(crash_count)?;
            warn!(crash_count, "Sentinel file found from prior launch. Engine may have crashed abnormally.");
        }

        let mode = if crash_count >= self.max_engine_crashes {
            warn!(crash_count, max = self.max_engine_crashes, "Entering Safe Mode due to repeated abnormal shutdowns.");
            LaunchMode::SafeMode {
                reason: format!("Engine crashed {} consecutive times", crash_count),
            }
        } else {
            LaunchMode::Normal
        };

        // Create current session sentinel
        self.arm_sentinel()?;

        Ok(mode)
    }

    pub fn arm_sentinel(&self) -> Result<()> {
        if let Some(parent) = self.sentinel_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.sentinel_path, format!("{}", std::process::id()))?;
        Ok(())
    }

    /// Disarms sentinel on graceful engine shutdown.
    pub fn disarm_sentinel(&self) -> Result<()> {
        if self.sentinel_path.exists() {
            let _ = std::fs::remove_file(&self.sentinel_path);
        }
        // Reset consecutive crash counter on clean shutdown
        self.reset_crash_counter()?;
        info!("Sentinel disarmed cleanly. Crash counter reset.");
        Ok(())
    }

    fn read_crash_counter(&self) -> u32 {
        if !self.counter_path.exists() {
            return 0;
        }
        std::fs::read_to_string(&self.counter_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    fn write_crash_counter(&self, count: u32) -> Result<()> {
        if let Some(parent) = self.counter_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.counter_path, count.to_string())?;
        Ok(())
    }

    pub fn reset_crash_counter(&self) -> Result<()> {
        if self.counter_path.exists() {
            let _ = std::fs::remove_file(&self.counter_path);
        }
        Ok(())
    }
}
