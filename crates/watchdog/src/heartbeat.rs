use serde::{Deserialize, Serialize};

/// Heartbeat status payload received over named pipe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeartbeatPayload {
    pub engine_pid: u32,
    pub timestamp_ms: u64,
    pub uptime_secs: u64,
}

/// Tracks heartbeat history for the engine process.
#[derive(Debug, Clone)]
pub struct WatchdogHeartbeat {
    pub engine_pid: Option<u32>,
    pub last_heartbeat_ms: u64,
    pub timeout_ms: u64,
}

impl WatchdogHeartbeat {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            engine_pid: None,
            last_heartbeat_ms: 0,
            timeout_ms,
        }
    }

    pub fn record(&mut self, pid: u32, now_ms: u64) {
        self.engine_pid = Some(pid);
        self.last_heartbeat_ms = now_ms;
    }

    pub fn is_healthy(&self, now_ms: u64) -> bool {
        if self.last_heartbeat_ms == 0 {
            return true; // Not started or initial state
        }
        now_ms.saturating_sub(self.last_heartbeat_ms) <= self.timeout_ms
    }
}
