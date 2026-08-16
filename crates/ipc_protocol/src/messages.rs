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
    /// Request detailed subsystem health information
    GetSubsystemHealth,
    /// Request engine diagnostics (PID, uptime, tick count, memory)
    GetDiagnostics,
    /// Toggle visibility of desktop widget overlay window
    ToggleDesktopWidget,
    /// Set persistent custom position (x, y) for a widget plugin
    SetWidgetPosition { widget_id: String, x: i32, y: i32 },
    /// Set position lock state for a widget (true = locked, false = drag enabled)
    SetWidgetLock { widget_id: String, locked: bool },
    /// Toggle position lock state for a widget
    ToggleWidgetLock { widget_id: String },
    /// Query crash history for a specific widget or all widgets
    GetCrashHistory { widget_id: Option<String> },
    /// Request widget rollback to previous version
    RollbackWidget { widget_id: String },
    /// Query current system launch mode (Normal vs Safe Mode)
    GetLaunchMode,
    /// Exit Safe Mode and clear consecutive crash counter
    ExitSafeMode,
    /// Query list of currently quarantined widgets
    GetQuarantineList,
    /// Release a widget from quarantine
    ReleaseQuarantine { widget_id: String },
    /// Create a new desktop snapshot
    CreateSnapshot { name: String },
    /// Query list of all saved desktop snapshots
    ListSnapshots,
    /// Restore desktop layout and settings from snapshot ID
    RestoreSnapshot { snapshot_id: String },
    /// Delete a desktop snapshot by ID
    DeleteSnapshot { snapshot_id: String },
    /// Export snapshot by ID to specified path
    ExportSnapshot { snapshot_id: String, path: String },
    /// Import snapshot from specified path
    ImportSnapshot { path: String },
    /// Request capability access token for a widget
    RequestCapabilityToken { widget_id: String, capability: String },
    /// Revoke an active capability access token
    RevokeCapabilityToken { token_id: String },
    /// Query proactive resource and memory usage report for a widget
    GetWidgetResourceUsage { widget_id: String },
    /// Query persistent user capability grants
    GetCapabilityGrants { widget_id: Option<String> },
    /// Query structured health report for all engine subsystems
    GetHealthReport,
    /// Query current status of watchdog process
    GetWatchdogStatus,
    /// Start recording engine events
    StartRecording,
    /// Stop recording engine events
    StopRecording,
    /// Query recorded system event stream
    GetRecording,
    /// Replay recorded system event stream
    ReplayRecording { from_seq: u64 },
    /// Inject simulated failure for chaos testing
    InjectChaosFailure { scenario: String },
    /// Query status of observability platform
    GetObservabilityStatus,
    /// Query formatted Prometheus text exposition metrics
    GetPrometheusMetrics,
    /// Manually generate a crash minidump (.dmp) file
    GenerateMinidump { reason: String },
    /// List all generated minidump file paths
    ListMinidumps,
    /// Query current status of adaptive tick rate advisor & frame scheduler
    GetSchedulerStatus,
    /// Enable or disable adaptive tick rate advisor
    SetAdaptiveTickMode { enabled: bool },
    /// Query resource cache usage statistics
    GetResourceCacheStats,
    /// Inspect widget DOM layout bounds, draw commands, and memory footprint
    InspectWidget { widget_id: String },
    /// Toggle developer layout grid overlay
    ToggleLayoutGrid { enabled: bool },
    /// Hot-reload target widget in-place
    HotReloadWidget { widget_id: String },
    /// Synthesize widget manifest & Lua script from natural language prompt
    SynthesizeWidget { prompt: String },
    /// Generate matching color theme from desktop wallpaper
    GenerateWallpaperTheme { wallpaper_path: Option<String> },
    /// Query AI performance advice & repair suggestions for widgets
    GetAiPerformanceAdvice { widget_id: Option<String> },
    /// Search marketplace packages by query
    SearchMarketplace { query: String },
    /// Query current Group Policy rules
    GetEnterprisePolicy,
    /// Update Group Policy rules JSON
    UpdateEnterprisePolicy { policy_json: String },
    /// Query cryptographic append-only audit trail logs
    GetAuditLogs,
    /// Verify SHA-256 block hash integrity of audit chain
    VerifyAuditChain,
    /// Dynamically update target widget rendering & display config
    SetWidgetRenderConfig { widget_id: String, config_json: String },
    /// Query target widget rendering & display config
    GetWidgetRenderConfig { widget_id: String },
    /// Switch active desktop profile by ID
    SetDesktopProfile { profile_id: String },
    /// Query active desktop profile details
    GetActiveProfile,
    /// List all available desktop profiles
    ListProfiles,
    /// Resolve current theme design tokens JSON
    ResolveDesignTokens { theme_id: Option<String> },
    /// Synthesize complete desktop layout and theme from natural language prompt
    SynthesizeDesktop { prompt: String },
    /// Query extended 7.4 widget inspector report
    GetExtendedWidgetInspector { widget_id: String },
    /// Update global platform accessibility overrides
    SetAccessibilityMode {
        high_contrast: Option<bool>,
        reduce_motion: Option<bool>,
        reduce_transparency: Option<bool>,
        large_text: Option<bool>,
    },
    /// Recursively scan directories for widget.toml manifests and return metadata list
    DiscoverWidgets { search_paths: Option<Vec<String>> },
}

/// Metadata payload for a discovered widget plugin manifest scanned from disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredWidgetInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub update_interval_ms: u64,
    pub manifest_path: String,
    pub folder_path: String,
    pub is_loaded: bool,
    pub is_locked: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub target_fps: u32,
    pub description: String,
}

/// Telemetry metrics payload exchanged via Shared Memory Ring Buffer
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MetricPayload {
    pub timestamp_ms: u64,
    pub cpu_usage_pct: f32,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub gpu_usage_pct: f32,
    pub net_recv_bytes_per_sec: u64,
    pub net_sent_bytes_per_sec: u64,
    // Process & Application Metrics
    pub open_apps_count: u32,
    pub browser_tabs_count: u32,
    pub audio_playing_apps_count: u32,
    pub gaming_apps_count: u32,
    pub dev_suite_apps_count: u32,
    pub other_apps_count: u32,
    // Power & Audio Metrics
    pub master_volume_pct: f32,
    pub is_muted: bool,
    pub battery_charge_pct: f32,
    pub battery_remaining_secs: u64,
    pub is_charging: bool,
    // Multi-GPU & Display Topology Metrics
    pub total_gpu_count: u32,
    pub integrated_gpu_count: u32,
    pub dedicated_gpu_count: u32,
    pub total_display_count: u32,
    pub external_display_count: u32,
    pub virtual_display_count: u32,
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
            open_apps_count: 5,
            browser_tabs_count: 12,
            audio_playing_apps_count: 1,
            gaming_apps_count: 0,
            dev_suite_apps_count: 2,
            other_apps_count: 2,
            master_volume_pct: 75.0,
            is_muted: false,
            battery_charge_pct: 85.0,
            battery_remaining_secs: 14400,
            is_charging: true,
            total_gpu_count: 2,
            integrated_gpu_count: 1,
            dedicated_gpu_count: 1,
            total_display_count: 2,
            external_display_count: 1,
            virtual_display_count: 0,
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

    #[test]
    fn test_get_subsystem_health_serialization() {
        let cmd = ControlCommand::GetSubsystemHealth;
        let json = serde_json::to_string(&cmd).unwrap();
        assert_eq!(json, "\"GetSubsystemHealth\"");
        let decoded: ControlCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_get_diagnostics_serialization() {
        let cmd = ControlCommand::GetDiagnostics;
        let json = serde_json::to_string(&cmd).unwrap();
        assert_eq!(json, "\"GetDiagnostics\"");
        let decoded: ControlCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_toggle_desktop_widget_serialization() {
        let cmd = ControlCommand::ToggleDesktopWidget;
        let json = serde_json::to_string(&cmd).unwrap();
        assert_eq!(json, "\"ToggleDesktopWidget\"");
        let decoded: ControlCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, cmd);
    }

    #[test]
    fn test_widget_position_and_lock_serialization() {
        let pos_cmd = ControlCommand::SetWidgetPosition {
            widget_id: "perf_monitor_widget".to_string(),
            x: 450,
            y: 220,
        };
        let json = serde_json::to_string(&pos_cmd).unwrap();
        let decoded: ControlCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, pos_cmd);

        let lock_cmd = ControlCommand::SetWidgetLock {
            widget_id: "perf_monitor_widget".to_string(),
            locked: true,
        };
        let json2 = serde_json::to_string(&lock_cmd).unwrap();
        let decoded2: ControlCommand = serde_json::from_str(&json2).unwrap();
        assert_eq!(decoded2, lock_cmd);

        let toggle_cmd = ControlCommand::ToggleWidgetLock {
            widget_id: "perf_monitor_widget".to_string(),
        };
        let json3 = serde_json::to_string(&toggle_cmd).unwrap();
        let decoded3: ControlCommand = serde_json::from_str(&json3).unwrap();
        assert_eq!(decoded3, toggle_cmd);
    }

    #[test]
    fn test_74_control_command_serialization() {
        let cmd = ControlCommand::SetDesktopProfile {
            profile_id: "profile.gaming".to_string(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: ControlCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, cmd);

        let synth_cmd = ControlCommand::SynthesizeDesktop {
            prompt: "cyberpunk workstation".to_string(),
        };
        let json_synth = serde_json::to_string(&synth_cmd).unwrap();
        let decoded_synth: ControlCommand = serde_json::from_str(&json_synth).unwrap();
        assert_eq!(decoded_synth, synth_cmd);
    }

    #[test]
    fn test_marketplace_snapshot_security_serialization() {
        let search_cmd = ControlCommand::SearchMarketplace {
            query: "monitoring".to_string(),
        };
        let json_search = serde_json::to_string(&search_cmd).unwrap();
        let decoded_search: ControlCommand = serde_json::from_str(&json_search).unwrap();
        assert_eq!(decoded_search, search_cmd);

        let snap_cmd = ControlCommand::CreateSnapshot {
            name: "Baseline".to_string(),
        };
        let json_snap = serde_json::to_string(&snap_cmd).unwrap();
        let decoded_snap: ControlCommand = serde_json::from_str(&json_snap).unwrap();
        assert_eq!(decoded_snap, snap_cmd);

        let list_snap = ControlCommand::ListSnapshots;
        let json_list = serde_json::to_string(&list_snap).unwrap();
        let decoded_list: ControlCommand = serde_json::from_str(&json_list).unwrap();
        assert_eq!(decoded_list, list_snap);

        let restore_snap = ControlCommand::RestoreSnapshot {
            snapshot_id: "snap-01".to_string(),
        };
        let json_restore = serde_json::to_string(&restore_snap).unwrap();
        let decoded_restore: ControlCommand = serde_json::from_str(&json_restore).unwrap();
        assert_eq!(decoded_restore, restore_snap);

        let delete_snap = ControlCommand::DeleteSnapshot {
            snapshot_id: "snap-01".to_string(),
        };
        let json_del = serde_json::to_string(&delete_snap).unwrap();
        let decoded_del: ControlCommand = serde_json::from_str(&json_del).unwrap();
        assert_eq!(decoded_del, delete_snap);
    }
}

