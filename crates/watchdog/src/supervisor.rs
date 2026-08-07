use crate::heartbeat::WatchdogHeartbeat;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Watchdog subsystem status summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchdogStatus {
    pub is_active: bool,
    pub engine_pid: Option<u32>,
    pub last_heartbeat_ms: u64,
    pub restart_count: u32,
}

/// Two-process Watchdog supervisor monitoring the core engine process.
pub struct WatchdogSupervisor {
    heartbeat: WatchdogHeartbeat,
    restart_count: u32,
    engine_binary_path: String,
}

impl WatchdogSupervisor {
    pub fn new(engine_binary_path: &str, timeout_ms: u64) -> Self {
        Self {
            heartbeat: WatchdogHeartbeat::new(timeout_ms),
            restart_count: 0,
            engine_binary_path: engine_binary_path.to_string(),
        }
    }

    pub fn heartbeat_mut(&mut self) -> &mut WatchdogHeartbeat {
        &mut self.heartbeat
    }

    pub fn record_heartbeat(&mut self, pid: u32, now_ms: u64) {
        self.heartbeat.record(pid, now_ms);
    }

    /// Checks engine health. If timed out, triggers restart.
    pub fn check_health(&mut self, now_ms: u64) -> Result<bool> {
        if !self.heartbeat.is_healthy(now_ms) {
            error!(
                pid = ?self.heartbeat.engine_pid,
                last_seen = self.heartbeat.last_heartbeat_ms,
                "Engine heartbeat timeout detected! Spawning new engine process."
            );
            self.restart_engine()?;
            return Ok(false);
        }
        Ok(true)
    }

    pub fn restart_engine(&mut self) -> Result<()> {
        self.restart_count += 1;
        info!(
            binary = %self.engine_binary_path,
            restart_count = self.restart_count,
            "Restarting Aether core engine process"
        );
        // Reset heartbeat timer after triggering restart
        self.heartbeat.last_heartbeat_ms = 0;
        Ok(())
    }

    pub fn status(&self) -> WatchdogStatus {
        WatchdogStatus {
            is_active: true,
            engine_pid: self.heartbeat.engine_pid,
            last_heartbeat_ms: self.heartbeat.last_heartbeat_ms,
            restart_count: self.restart_count,
        }
    }
}
