use crate::heartbeat::WatchdogHeartbeat;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{error, info, warn};

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

        // Attempt actual process spawning if executable exists
        let path = std::path::Path::new(&self.engine_binary_path);
        if path.exists() {
            match Command::new(path).spawn() {
                Ok(child) => {
                    info!(
                        "Successfully spawned replacement engine process with PID {}",
                        child.id()
                    );
                    self.heartbeat.engine_pid = Some(child.id());
                }
                Err(e) => {
                    error!(
                        "Failed to spawn replacement engine process at {:?}: {}",
                        path, e
                    );
                }
            }
        } else {
            warn!(
                "Engine binary path '{:?}' not found on disk; registered restart intent #{}",
                path, self.restart_count
            );
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_supervisor_restart_logic() {
        let mut supervisor = WatchdogSupervisor::new("mock_engine.exe", 5000);
        supervisor.record_heartbeat(1234, 1000);
        assert!(supervisor.check_health(2000).unwrap());

        // Simulate heartbeat timeout at 7000ms (> 5000ms delta)
        assert!(!supervisor.check_health(7000).unwrap());
        assert_eq!(supervisor.status().restart_count, 1);
    }

    #[test]
    fn test_watchdog_real_process_spawning() {
        #[cfg(windows)]
        let exe = "C:\\Windows\\System32\\cmd.exe";
        #[cfg(not(windows))]
        let exe = "/bin/sh";

        if std::path::Path::new(exe).exists() {
            let mut supervisor = WatchdogSupervisor::new(exe, 1000);
            supervisor.record_heartbeat(9999, 1000);
            let result = supervisor.restart_engine();
            assert!(result.is_ok());
            assert!(supervisor.status().engine_pid.is_some());
        }
    }
}
