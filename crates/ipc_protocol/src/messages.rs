use serde::{Deserialize, Serialize};

/// High-level IPC Control Commands sent over Win32 Named Pipes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ControlCommand {
    /// Ping message to check engine readiness
    Ping,
    /// Response to Ping command
    Pong,
    /// Request engine health status and active widget metrics
    GetStatus,
    /// Load a widget from specified manifest path
    LoadWidget { manifest_path: String },
    /// Unload an active widget by ID
    UnloadWidget { widget_id: String },
    /// Set global theme mode ("light", "dark", "system")
    SetThemeMode { mode: String },
    /// Reload all loaded widgets
    ReloadAll,
}

/// Telemetry metrics payload exchanged via Shared Memory Ring Buffer
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MetricPayload {
    pub timestamp_ms: u64,
    pub cpu_usage_pct: f32,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub gpu_usage_pct: f32,
    pub net_recv_bytes_per_sec: u64,
    pub net_sent_bytes_per_sec: u64,
}

impl Default for MetricPayload {
    fn default() -> Self {
        Self {
            timestamp_ms: 0,
            cpu_usage_pct: 0.0,
            memory_used_mb: 0.0,
            memory_total_mb: 0.0,
            gpu_usage_pct: 0.0,
            net_recv_bytes_per_sec: 0,
            net_sent_bytes_per_sec: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_command_serialization() {
        let cmd = ControlCommand::LoadWidget {
            manifest_path: "C:\\Widgets\\Weather\\widget.toml".to_string(),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: ControlCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_metric_payload_default() {
        let metric = MetricPayload::default();
        assert_eq!(metric.timestamp_ms, 0);
        assert_eq!(metric.cpu_usage_pct, 0.0);
    }
}
