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
    /// Search marketplace packages by query, optionally filtered by category
    SearchMarketplace {
        query: String,
        #[serde(default)]
        category: Option<String>,
    },
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
    /// List all registered widgets with full descriptor (state + display config)
    ListWidgets,
    /// Update display options (opacity, scale, locked, enabled) for a specific widget
    UpdateWidgetDisplayOptions {
        widget_id: String,
        opacity: Option<f32>,
        scale: Option<f32>,
        locked: Option<bool>,
        enabled: Option<bool>,
    },
    /// Swap two widgets — either their desktop positions or their full configurations
    QuickSwapWidget {
        from_id: String,
        to_id: String,
        /// "position" or "configuration"
        mode: String,
    },
    /// Enable a previously disabled widget (sets visible + updating)
    EnableWidget { widget_id: String },
    /// Disable a widget without unloading it (invisible, stops updating)
    DisableWidget { widget_id: String },
    /// Set the opacity of a widget [0.0, 1.0]
    SetWidgetOpacity { widget_id: String, opacity: f32 },
    /// Reset a widget to its default display options
    ResetWidgetConfig { widget_id: String },
    /// Trigger manual or aggressive garbage collection and physical working set trimming
    TriggerGarbageCollection {
        #[serde(default)]
        aggressive: bool,
        #[serde(default)]
        trim_working_set: bool,
    },
    /// Set runtime garbage collection and development mode flags
    SetGarbageCollectionPolicy {
        dev_mode: bool,
        aggressive: bool,
        interval_secs: u64,
    },
    /// Query current garbage collection statistics and memory reclamation metrics
    GetGarbageCollectionStats,
}

/// Logical domain categorization for `ControlCommand` routing and telemetry auditing (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandDomain {
    Core,
    Widget,
    Theme,
    Diagnostics,
    Snapshot,
    Security,
    Observability,
    Marketplace,
    Ai,
    Enterprise,
    Profile,
    GarbageCollection,
}

impl ControlCommand {
    /// Classifies this `ControlCommand` into its logical `CommandDomain`.
    pub fn domain(&self) -> CommandDomain {
        match self {
            Self::Ping
            | Self::Pong
            | Self::GetStatus
            | Self::GetLaunchMode
            | Self::ExitSafeMode
            | Self::GetSchedulerStatus
            | Self::SetAdaptiveTickMode { .. } => CommandDomain::Core,

            Self::LoadWidget { .. }
            | Self::UnloadWidget { .. }
            | Self::ReloadAll
            | Self::ToggleDesktopWidget
            | Self::SetWidgetPosition { .. }
            | Self::SetWidgetLock { .. }
            | Self::ToggleWidgetLock { .. }
            | Self::RollbackWidget { .. }
            | Self::HotReloadWidget { .. }
            | Self::SetWidgetRenderConfig { .. }
            | Self::GetWidgetRenderConfig { .. }
            | Self::DiscoverWidgets { .. }
            | Self::ListWidgets
            | Self::UpdateWidgetDisplayOptions { .. }
            | Self::QuickSwapWidget { .. }
            | Self::EnableWidget { .. }
            | Self::DisableWidget { .. }
            | Self::SetWidgetOpacity { .. }
            | Self::ResetWidgetConfig { .. } => CommandDomain::Widget,

            Self::SetThemeMode { .. }
            | Self::ResolveDesignTokens { .. }
            | Self::SetAccessibilityMode { .. } => CommandDomain::Theme,

            Self::GetSubsystemHealth
            | Self::GetDiagnostics
            | Self::GetCrashHistory { .. }
            | Self::GetQuarantineList
            | Self::ReleaseQuarantine { .. }
            | Self::GetWidgetResourceUsage { .. }
            | Self::GetHealthReport
            | Self::GetWatchdogStatus
            | Self::InjectChaosFailure { .. }
            | Self::GetResourceCacheStats
            | Self::InspectWidget { .. }
            | Self::ToggleLayoutGrid { .. }
            | Self::GetExtendedWidgetInspector { .. } => CommandDomain::Diagnostics,

            Self::CreateSnapshot { .. }
            | Self::ListSnapshots
            | Self::RestoreSnapshot { .. }
            | Self::DeleteSnapshot { .. }
            | Self::ExportSnapshot { .. }
            | Self::ImportSnapshot { .. } => CommandDomain::Snapshot,

            Self::RequestCapabilityToken { .. }
            | Self::RevokeCapabilityToken { .. }
            | Self::GetCapabilityGrants { .. } => CommandDomain::Security,

            Self::StartRecording
            | Self::StopRecording
            | Self::GetRecording
            | Self::ReplayRecording { .. }
            | Self::GetObservabilityStatus
            | Self::GetPrometheusMetrics
            | Self::GenerateMinidump { .. }
            | Self::ListMinidumps => CommandDomain::Observability,

            Self::SearchMarketplace { .. } => CommandDomain::Marketplace,

            Self::SynthesizeWidget { .. }
            | Self::GenerateWallpaperTheme { .. }
            | Self::GetAiPerformanceAdvice { .. }
            | Self::SynthesizeDesktop { .. } => CommandDomain::Ai,

            Self::GetEnterprisePolicy
            | Self::UpdateEnterprisePolicy { .. }
            | Self::GetAuditLogs
            | Self::VerifyAuditChain => CommandDomain::Enterprise,

            Self::SetDesktopProfile { .. }
            | Self::GetActiveProfile
            | Self::ListProfiles => CommandDomain::Profile,

            Self::TriggerGarbageCollection { .. }
            | Self::SetGarbageCollectionPolicy { .. }
            | Self::GetGarbageCollectionStats => CommandDomain::GarbageCollection,
        }
    }

    /// Converts this flat `ControlCommand` into a modular domain-scoped `DomainCommand` (`FI-003` / `AC-002`).
    pub fn into_domain_command(&self) -> DomainCommand {
        match self.clone() {
            Self::Ping => DomainCommand::Core(CoreCommand::Ping),
            Self::Pong => DomainCommand::Core(CoreCommand::Pong),
            Self::GetStatus => DomainCommand::Core(CoreCommand::GetStatus),
            Self::GetLaunchMode => DomainCommand::Core(CoreCommand::GetLaunchMode),
            Self::ExitSafeMode => DomainCommand::Core(CoreCommand::ExitSafeMode),
            Self::GetSchedulerStatus => DomainCommand::Core(CoreCommand::GetSchedulerStatus),
            Self::SetAdaptiveTickMode { enabled } => {
                DomainCommand::Core(CoreCommand::SetAdaptiveTickMode { enabled })
            }

            Self::LoadWidget { manifest_path } => {
                DomainCommand::Widget(WidgetCommand::LoadWidget { manifest_path })
            }
            Self::UnloadWidget { widget_id } => {
                DomainCommand::Widget(WidgetCommand::UnloadWidget { widget_id })
            }
            Self::ReloadAll => DomainCommand::Widget(WidgetCommand::ReloadAll),
            Self::ToggleDesktopWidget => DomainCommand::Widget(WidgetCommand::ToggleDesktopWidget),
            Self::SetWidgetPosition { widget_id, x, y } => {
                DomainCommand::Widget(WidgetCommand::SetWidgetPosition { widget_id, x, y })
            }
            Self::SetWidgetLock { widget_id, locked } => {
                DomainCommand::Widget(WidgetCommand::SetWidgetLock { widget_id, locked })
            }
            Self::ToggleWidgetLock { widget_id } => {
                DomainCommand::Widget(WidgetCommand::ToggleWidgetLock { widget_id })
            }
            Self::RollbackWidget { widget_id } => {
                DomainCommand::Widget(WidgetCommand::RollbackWidget { widget_id })
            }
            Self::HotReloadWidget { widget_id } => {
                DomainCommand::Widget(WidgetCommand::HotReloadWidget { widget_id })
            }
            Self::SetWidgetRenderConfig {
                widget_id,
                config_json,
            } => DomainCommand::Widget(WidgetCommand::SetWidgetRenderConfig {
                widget_id,
                config_json,
            }),
            Self::GetWidgetRenderConfig { widget_id } => {
                DomainCommand::Widget(WidgetCommand::GetWidgetRenderConfig { widget_id })
            }
            Self::DiscoverWidgets { search_paths } => {
                DomainCommand::Widget(WidgetCommand::DiscoverWidgets { search_paths })
            }
            Self::ListWidgets => DomainCommand::Widget(WidgetCommand::ListWidgets),
            Self::UpdateWidgetDisplayOptions {
                widget_id,
                opacity,
                scale,
                locked,
                enabled,
            } => DomainCommand::Widget(WidgetCommand::UpdateWidgetDisplayOptions {
                widget_id,
                opacity,
                scale,
                locked,
                enabled,
            }),
            Self::QuickSwapWidget {
                from_id,
                to_id,
                mode,
            } => DomainCommand::Widget(WidgetCommand::QuickSwapWidget {
                from_id,
                to_id,
                mode,
            }),
            Self::EnableWidget { widget_id } => {
                DomainCommand::Widget(WidgetCommand::EnableWidget { widget_id })
            }
            Self::DisableWidget { widget_id } => {
                DomainCommand::Widget(WidgetCommand::DisableWidget { widget_id })
            }
            Self::SetWidgetOpacity { widget_id, opacity } => {
                DomainCommand::Widget(WidgetCommand::SetWidgetOpacity { widget_id, opacity })
            }
            Self::ResetWidgetConfig { widget_id } => {
                DomainCommand::Widget(WidgetCommand::ResetWidgetConfig { widget_id })
            }

            Self::SetThemeMode { mode } => DomainCommand::Theme(ThemeCommand::SetThemeMode { mode }),
            Self::ResolveDesignTokens { theme_id } => {
                DomainCommand::Theme(ThemeCommand::ResolveDesignTokens { theme_id })
            }
            Self::SetAccessibilityMode {
                high_contrast,
                reduce_motion,
                reduce_transparency,
                large_text,
            } => DomainCommand::Theme(ThemeCommand::SetAccessibilityMode {
                high_contrast,
                reduce_motion,
                reduce_transparency,
                large_text,
            }),

            Self::GetSubsystemHealth => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetSubsystemHealth)
            }
            Self::GetDiagnostics => DomainCommand::Diagnostics(DiagnosticsCommand::GetDiagnostics),
            Self::GetCrashHistory { widget_id } => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetCrashHistory { widget_id })
            }
            Self::GetQuarantineList => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetQuarantineList)
            }
            Self::ReleaseQuarantine { widget_id } => {
                DomainCommand::Diagnostics(DiagnosticsCommand::ReleaseQuarantine { widget_id })
            }
            Self::GetWidgetResourceUsage { widget_id } => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetWidgetResourceUsage { widget_id })
            }
            Self::GetHealthReport => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetHealthReport)
            }
            Self::GetWatchdogStatus => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetWatchdogStatus)
            }
            Self::InjectChaosFailure { scenario } => {
                DomainCommand::Diagnostics(DiagnosticsCommand::InjectChaosFailure { scenario })
            }
            Self::GetResourceCacheStats => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetResourceCacheStats)
            }
            Self::InspectWidget { widget_id } => {
                DomainCommand::Diagnostics(DiagnosticsCommand::InspectWidget { widget_id })
            }
            Self::ToggleLayoutGrid { enabled } => {
                DomainCommand::Diagnostics(DiagnosticsCommand::ToggleLayoutGrid { enabled })
            }
            Self::GetExtendedWidgetInspector { widget_id } => {
                DomainCommand::Diagnostics(DiagnosticsCommand::GetExtendedWidgetInspector {
                    widget_id,
                })
            }

            Self::CreateSnapshot { name } => {
                DomainCommand::Snapshot(SnapshotCommand::CreateSnapshot { name })
            }
            Self::ListSnapshots => DomainCommand::Snapshot(SnapshotCommand::ListSnapshots),
            Self::RestoreSnapshot { snapshot_id } => {
                DomainCommand::Snapshot(SnapshotCommand::RestoreSnapshot { snapshot_id })
            }
            Self::DeleteSnapshot { snapshot_id } => {
                DomainCommand::Snapshot(SnapshotCommand::DeleteSnapshot { snapshot_id })
            }
            Self::ExportSnapshot { snapshot_id, path } => {
                DomainCommand::Snapshot(SnapshotCommand::ExportSnapshot { snapshot_id, path })
            }
            Self::ImportSnapshot { path } => {
                DomainCommand::Snapshot(SnapshotCommand::ImportSnapshot { path })
            }

            Self::RequestCapabilityToken {
                widget_id,
                capability,
            } => DomainCommand::Security(SecurityCommand::RequestCapabilityToken {
                widget_id,
                capability,
            }),
            Self::RevokeCapabilityToken { token_id } => {
                DomainCommand::Security(SecurityCommand::RevokeCapabilityToken { token_id })
            }
            Self::GetCapabilityGrants { widget_id } => {
                DomainCommand::Security(SecurityCommand::GetCapabilityGrants { widget_id })
            }

            Self::StartRecording => {
                DomainCommand::Observability(ObservabilityCommand::StartRecording)
            }
            Self::StopRecording => {
                DomainCommand::Observability(ObservabilityCommand::StopRecording)
            }
            Self::GetRecording => DomainCommand::Observability(ObservabilityCommand::GetRecording),
            Self::ReplayRecording { from_seq } => {
                DomainCommand::Observability(ObservabilityCommand::ReplayRecording { from_seq })
            }
            Self::GetObservabilityStatus => {
                DomainCommand::Observability(ObservabilityCommand::GetObservabilityStatus)
            }
            Self::GetPrometheusMetrics => {
                DomainCommand::Observability(ObservabilityCommand::GetPrometheusMetrics)
            }
            Self::GenerateMinidump { reason } => {
                DomainCommand::Observability(ObservabilityCommand::GenerateMinidump { reason })
            }
            Self::ListMinidumps => {
                DomainCommand::Observability(ObservabilityCommand::ListMinidumps)
            }

            Self::SearchMarketplace { query, category } => {
                DomainCommand::Marketplace(MarketplaceCommand::SearchMarketplace { query, category })
            }

            Self::SynthesizeWidget { prompt } => {
                DomainCommand::Ai(AiCommand::SynthesizeWidget { prompt })
            }
            Self::GenerateWallpaperTheme { wallpaper_path } => {
                DomainCommand::Ai(AiCommand::GenerateWallpaperTheme { wallpaper_path })
            }
            Self::GetAiPerformanceAdvice { widget_id } => {
                DomainCommand::Ai(AiCommand::GetAiPerformanceAdvice { widget_id })
            }
            Self::SynthesizeDesktop { prompt } => {
                DomainCommand::Ai(AiCommand::SynthesizeDesktop { prompt })
            }

            Self::GetEnterprisePolicy => {
                DomainCommand::Enterprise(EnterpriseCommand::GetEnterprisePolicy)
            }
            Self::UpdateEnterprisePolicy { policy_json } => {
                DomainCommand::Enterprise(EnterpriseCommand::UpdateEnterprisePolicy { policy_json })
            }
            Self::GetAuditLogs => DomainCommand::Enterprise(EnterpriseCommand::GetAuditLogs),
            Self::VerifyAuditChain => {
                DomainCommand::Enterprise(EnterpriseCommand::VerifyAuditChain)
            }

            Self::SetDesktopProfile { profile_id } => {
                DomainCommand::Profile(ProfileCommand::SetDesktopProfile { profile_id })
            }
            Self::GetActiveProfile => DomainCommand::Profile(ProfileCommand::GetActiveProfile),
            Self::ListProfiles => DomainCommand::Profile(ProfileCommand::ListProfiles),

            Self::TriggerGarbageCollection {
                aggressive,
                trim_working_set,
            } => DomainCommand::GarbageCollection(
                GarbageCollectionCommand::TriggerGarbageCollection {
                    aggressive,
                    trim_working_set,
                },
            ),
            Self::SetGarbageCollectionPolicy {
                dev_mode,
                aggressive,
                interval_secs,
            } => DomainCommand::GarbageCollection(
                GarbageCollectionCommand::SetGarbageCollectionPolicy {
                    dev_mode,
                    aggressive,
                    interval_secs,
                },
            ),
            Self::GetGarbageCollectionStats => DomainCommand::GarbageCollection(
                GarbageCollectionCommand::GetGarbageCollectionStats,
            ),
        }
    }
}

/// Core daemon and scheduler domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoreCommand {
    Ping,
    Pong,
    GetStatus,
    GetLaunchMode,
    ExitSafeMode,
    GetSchedulerStatus,
    SetAdaptiveTickMode { enabled: bool },
}

/// Widget lifecycle, positioning, and display configuration domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WidgetCommand {
    LoadWidget { manifest_path: String },
    UnloadWidget { widget_id: String },
    ReloadAll,
    ToggleDesktopWidget,
    SetWidgetPosition { widget_id: String, x: i32, y: i32 },
    SetWidgetLock { widget_id: String, locked: bool },
    ToggleWidgetLock { widget_id: String },
    RollbackWidget { widget_id: String },
    HotReloadWidget { widget_id: String },
    SetWidgetRenderConfig { widget_id: String, config_json: String },
    GetWidgetRenderConfig { widget_id: String },
    DiscoverWidgets { search_paths: Option<Vec<String>> },
    ListWidgets,
    UpdateWidgetDisplayOptions {
        widget_id: String,
        opacity: Option<f32>,
        scale: Option<f32>,
        locked: Option<bool>,
        enabled: Option<bool>,
    },
    QuickSwapWidget {
        from_id: String,
        to_id: String,
        mode: String,
    },
    EnableWidget { widget_id: String },
    DisableWidget { widget_id: String },
    SetWidgetOpacity { widget_id: String, opacity: f32 },
    ResetWidgetConfig { widget_id: String },
}

/// Theme, design token, and accessibility domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeCommand {
    SetThemeMode { mode: String },
    ResolveDesignTokens { theme_id: Option<String> },
    SetAccessibilityMode {
        high_contrast: Option<bool>,
        reduce_motion: Option<bool>,
        reduce_transparency: Option<bool>,
        large_text: Option<bool>,
    },
}

/// Diagnostics, health, quarantine, and inspector domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticsCommand {
    GetSubsystemHealth,
    GetDiagnostics,
    GetCrashHistory { widget_id: Option<String> },
    GetQuarantineList,
    ReleaseQuarantine { widget_id: String },
    GetWidgetResourceUsage { widget_id: String },
    GetHealthReport,
    GetWatchdogStatus,
    InjectChaosFailure { scenario: String },
    GetResourceCacheStats,
    InspectWidget { widget_id: String },
    ToggleLayoutGrid { enabled: bool },
    GetExtendedWidgetInspector { widget_id: String },
}

/// Transactional snapshot domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapshotCommand {
    CreateSnapshot { name: String },
    ListSnapshots,
    RestoreSnapshot { snapshot_id: String },
    DeleteSnapshot { snapshot_id: String },
    ExportSnapshot { snapshot_id: String, path: String },
    ImportSnapshot { path: String },
}

/// Capability token and sandbox security domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityCommand {
    RequestCapabilityToken { widget_id: String, capability: String },
    RevokeCapabilityToken { token_id: String },
    GetCapabilityGrants { widget_id: Option<String> },
}

/// Event stream recording, Prometheus, and minidump observability domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObservabilityCommand {
    StartRecording,
    StopRecording,
    GetRecording,
    ReplayRecording { from_seq: u64 },
    GetObservabilityStatus,
    GetPrometheusMetrics,
    GenerateMinidump { reason: String },
    ListMinidumps,
}

/// Marketplace search and catalog domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarketplaceCommand {
    SearchMarketplace {
        query: String,
        category: Option<String>,
    },
}

/// AI synthesis, wallpaper theme generation, and advisor domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiCommand {
    SynthesizeWidget { prompt: String },
    GenerateWallpaperTheme { wallpaper_path: Option<String> },
    GetAiPerformanceAdvice { widget_id: Option<String> },
    SynthesizeDesktop { prompt: String },
}

/// Enterprise policy and SHA-256 audit log domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnterpriseCommand {
    GetEnterprisePolicy,
    UpdateEnterprisePolicy { policy_json: String },
    GetAuditLogs,
    VerifyAuditChain,
}

/// Desktop activity profile domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileCommand {
    SetDesktopProfile { profile_id: String },
    GetActiveProfile,
    ListProfiles,
}

/// Memory reclamation and garbage collection domain commands (`FI-003` / `AC-002`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GarbageCollectionCommand {
    TriggerGarbageCollection {
        aggressive: bool,
        trim_working_set: bool,
    },
    SetGarbageCollectionPolicy {
        dev_mode: bool,
        aggressive: bool,
        interval_secs: u64,
    },
    GetGarbageCollectionStats,
}

/// Hierarchical domain-grouped command wrapper (`FI-003` / `AC-002`) with lossless conversion to/from `ControlCommand`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainCommand {
    Core(CoreCommand),
    Widget(WidgetCommand),
    Theme(ThemeCommand),
    Diagnostics(DiagnosticsCommand),
    Snapshot(SnapshotCommand),
    Security(SecurityCommand),
    Observability(ObservabilityCommand),
    Marketplace(MarketplaceCommand),
    Ai(AiCommand),
    Enterprise(EnterpriseCommand),
    Profile(ProfileCommand),
    GarbageCollection(GarbageCollectionCommand),
}

impl DomainCommand {
    /// Returns the `CommandDomain` discriminant for this `DomainCommand`.
    pub fn domain(&self) -> CommandDomain {
        match self {
            Self::Core(_) => CommandDomain::Core,
            Self::Widget(_) => CommandDomain::Widget,
            Self::Theme(_) => CommandDomain::Theme,
            Self::Diagnostics(_) => CommandDomain::Diagnostics,
            Self::Snapshot(_) => CommandDomain::Snapshot,
            Self::Security(_) => CommandDomain::Security,
            Self::Observability(_) => CommandDomain::Observability,
            Self::Marketplace(_) => CommandDomain::Marketplace,
            Self::Ai(_) => CommandDomain::Ai,
            Self::Enterprise(_) => CommandDomain::Enterprise,
            Self::Profile(_) => CommandDomain::Profile,
            Self::GarbageCollection(_) => CommandDomain::GarbageCollection,
        }
    }

    /// Converts this modular `DomainCommand` back into the wire-compatible `ControlCommand`.
    pub fn into_control_command(self) -> ControlCommand {
        match self {
            Self::Core(c) => match c {
                CoreCommand::Ping => ControlCommand::Ping,
                CoreCommand::Pong => ControlCommand::Pong,
                CoreCommand::GetStatus => ControlCommand::GetStatus,
                CoreCommand::GetLaunchMode => ControlCommand::GetLaunchMode,
                CoreCommand::ExitSafeMode => ControlCommand::ExitSafeMode,
                CoreCommand::GetSchedulerStatus => ControlCommand::GetSchedulerStatus,
                CoreCommand::SetAdaptiveTickMode { enabled } => {
                    ControlCommand::SetAdaptiveTickMode { enabled }
                }
            },
            Self::Widget(w) => match w {
                WidgetCommand::LoadWidget { manifest_path } => {
                    ControlCommand::LoadWidget { manifest_path }
                }
                WidgetCommand::UnloadWidget { widget_id } => {
                    ControlCommand::UnloadWidget { widget_id }
                }
                WidgetCommand::ReloadAll => ControlCommand::ReloadAll,
                WidgetCommand::ToggleDesktopWidget => ControlCommand::ToggleDesktopWidget,
                WidgetCommand::SetWidgetPosition { widget_id, x, y } => {
                    ControlCommand::SetWidgetPosition { widget_id, x, y }
                }
                WidgetCommand::SetWidgetLock { widget_id, locked } => {
                    ControlCommand::SetWidgetLock { widget_id, locked }
                }
                WidgetCommand::ToggleWidgetLock { widget_id } => {
                    ControlCommand::ToggleWidgetLock { widget_id }
                }
                WidgetCommand::RollbackWidget { widget_id } => {
                    ControlCommand::RollbackWidget { widget_id }
                }
                WidgetCommand::HotReloadWidget { widget_id } => {
                    ControlCommand::HotReloadWidget { widget_id }
                }
                WidgetCommand::SetWidgetRenderConfig {
                    widget_id,
                    config_json,
                } => ControlCommand::SetWidgetRenderConfig {
                    widget_id,
                    config_json,
                },
                WidgetCommand::GetWidgetRenderConfig { widget_id } => {
                    ControlCommand::GetWidgetRenderConfig { widget_id }
                }
                WidgetCommand::DiscoverWidgets { search_paths } => {
                    ControlCommand::DiscoverWidgets { search_paths }
                }
                WidgetCommand::ListWidgets => ControlCommand::ListWidgets,
                WidgetCommand::UpdateWidgetDisplayOptions {
                    widget_id,
                    opacity,
                    scale,
                    locked,
                    enabled,
                } => ControlCommand::UpdateWidgetDisplayOptions {
                    widget_id,
                    opacity,
                    scale,
                    locked,
                    enabled,
                },
                WidgetCommand::QuickSwapWidget {
                    from_id,
                    to_id,
                    mode,
                } => ControlCommand::QuickSwapWidget {
                    from_id,
                    to_id,
                    mode,
                },
                WidgetCommand::EnableWidget { widget_id } => {
                    ControlCommand::EnableWidget { widget_id }
                }
                WidgetCommand::DisableWidget { widget_id } => {
                    ControlCommand::DisableWidget { widget_id }
                }
                WidgetCommand::SetWidgetOpacity { widget_id, opacity } => {
                    ControlCommand::SetWidgetOpacity { widget_id, opacity }
                }
                WidgetCommand::ResetWidgetConfig { widget_id } => {
                    ControlCommand::ResetWidgetConfig { widget_id }
                }
            },
            Self::Theme(t) => match t {
                ThemeCommand::SetThemeMode { mode } => ControlCommand::SetThemeMode { mode },
                ThemeCommand::ResolveDesignTokens { theme_id } => {
                    ControlCommand::ResolveDesignTokens { theme_id }
                }
                ThemeCommand::SetAccessibilityMode {
                    high_contrast,
                    reduce_motion,
                    reduce_transparency,
                    large_text,
                } => ControlCommand::SetAccessibilityMode {
                    high_contrast,
                    reduce_motion,
                    reduce_transparency,
                    large_text,
                },
            },
            Self::Diagnostics(d) => match d {
                DiagnosticsCommand::GetSubsystemHealth => ControlCommand::GetSubsystemHealth,
                DiagnosticsCommand::GetDiagnostics => ControlCommand::GetDiagnostics,
                DiagnosticsCommand::GetCrashHistory { widget_id } => {
                    ControlCommand::GetCrashHistory { widget_id }
                }
                DiagnosticsCommand::GetQuarantineList => ControlCommand::GetQuarantineList,
                DiagnosticsCommand::ReleaseQuarantine { widget_id } => {
                    ControlCommand::ReleaseQuarantine { widget_id }
                }
                DiagnosticsCommand::GetWidgetResourceUsage { widget_id } => {
                    ControlCommand::GetWidgetResourceUsage { widget_id }
                }
                DiagnosticsCommand::GetHealthReport => ControlCommand::GetHealthReport,
                DiagnosticsCommand::GetWatchdogStatus => ControlCommand::GetWatchdogStatus,
                DiagnosticsCommand::InjectChaosFailure { scenario } => {
                    ControlCommand::InjectChaosFailure { scenario }
                }
                DiagnosticsCommand::GetResourceCacheStats => ControlCommand::GetResourceCacheStats,
                DiagnosticsCommand::InspectWidget { widget_id } => {
                    ControlCommand::InspectWidget { widget_id }
                }
                DiagnosticsCommand::ToggleLayoutGrid { enabled } => {
                    ControlCommand::ToggleLayoutGrid { enabled }
                }
                DiagnosticsCommand::GetExtendedWidgetInspector { widget_id } => {
                    ControlCommand::GetExtendedWidgetInspector { widget_id }
                }
            },
            Self::Snapshot(s) => match s {
                SnapshotCommand::CreateSnapshot { name } => {
                    ControlCommand::CreateSnapshot { name }
                }
                SnapshotCommand::ListSnapshots => ControlCommand::ListSnapshots,
                SnapshotCommand::RestoreSnapshot { snapshot_id } => {
                    ControlCommand::RestoreSnapshot { snapshot_id }
                }
                SnapshotCommand::DeleteSnapshot { snapshot_id } => {
                    ControlCommand::DeleteSnapshot { snapshot_id }
                }
                SnapshotCommand::ExportSnapshot { snapshot_id, path } => {
                    ControlCommand::ExportSnapshot { snapshot_id, path }
                }
                SnapshotCommand::ImportSnapshot { path } => {
                    ControlCommand::ImportSnapshot { path }
                }
            },
            Self::Security(s) => match s {
                SecurityCommand::RequestCapabilityToken {
                    widget_id,
                    capability,
                } => ControlCommand::RequestCapabilityToken {
                    widget_id,
                    capability,
                },
                SecurityCommand::RevokeCapabilityToken { token_id } => {
                    ControlCommand::RevokeCapabilityToken { token_id }
                }
                SecurityCommand::GetCapabilityGrants { widget_id } => {
                    ControlCommand::GetCapabilityGrants { widget_id }
                }
            },
            Self::Observability(o) => match o {
                ObservabilityCommand::StartRecording => ControlCommand::StartRecording,
                ObservabilityCommand::StopRecording => ControlCommand::StopRecording,
                ObservabilityCommand::GetRecording => ControlCommand::GetRecording,
                ObservabilityCommand::ReplayRecording { from_seq } => {
                    ControlCommand::ReplayRecording { from_seq }
                }
                ObservabilityCommand::GetObservabilityStatus => {
                    ControlCommand::GetObservabilityStatus
                }
                ObservabilityCommand::GetPrometheusMetrics => ControlCommand::GetPrometheusMetrics,
                ObservabilityCommand::GenerateMinidump { reason } => {
                    ControlCommand::GenerateMinidump { reason }
                }
                ObservabilityCommand::ListMinidumps => ControlCommand::ListMinidumps,
            },
            Self::Marketplace(m) => match m {
                MarketplaceCommand::SearchMarketplace { query, category } => {
                    ControlCommand::SearchMarketplace { query, category }
                }
            },
            Self::Ai(a) => match a {
                AiCommand::SynthesizeWidget { prompt } => {
                    ControlCommand::SynthesizeWidget { prompt }
                }
                AiCommand::GenerateWallpaperTheme { wallpaper_path } => {
                    ControlCommand::GenerateWallpaperTheme { wallpaper_path }
                }
                AiCommand::GetAiPerformanceAdvice { widget_id } => {
                    ControlCommand::GetAiPerformanceAdvice { widget_id }
                }
                AiCommand::SynthesizeDesktop { prompt } => {
                    ControlCommand::SynthesizeDesktop { prompt }
                }
            },
            Self::Enterprise(e) => match e {
                EnterpriseCommand::GetEnterprisePolicy => ControlCommand::GetEnterprisePolicy,
                EnterpriseCommand::UpdateEnterprisePolicy { policy_json } => {
                    ControlCommand::UpdateEnterprisePolicy { policy_json }
                }
                EnterpriseCommand::GetAuditLogs => ControlCommand::GetAuditLogs,
                EnterpriseCommand::VerifyAuditChain => ControlCommand::VerifyAuditChain,
            },
            Self::Profile(p) => match p {
                ProfileCommand::SetDesktopProfile { profile_id } => {
                    ControlCommand::SetDesktopProfile { profile_id }
                }
                ProfileCommand::GetActiveProfile => ControlCommand::GetActiveProfile,
                ProfileCommand::ListProfiles => ControlCommand::ListProfiles,
            },
            Self::GarbageCollection(gc) => match gc {
                GarbageCollectionCommand::TriggerGarbageCollection {
                    aggressive,
                    trim_working_set,
                } => ControlCommand::TriggerGarbageCollection {
                    aggressive,
                    trim_working_set,
                },
                GarbageCollectionCommand::SetGarbageCollectionPolicy {
                    dev_mode,
                    aggressive,
                    interval_secs,
                } => ControlCommand::SetGarbageCollectionPolicy {
                    dev_mode,
                    aggressive,
                    interval_secs,
                },
                GarbageCollectionCommand::GetGarbageCollectionStats => {
                    ControlCommand::GetGarbageCollectionStats
                }
            },
        }
    }
}

impl From<ControlCommand> for DomainCommand {
    fn from(cmd: ControlCommand) -> Self {
        cmd.into_domain_command()
    }
}

impl From<DomainCommand> for ControlCommand {
    fn from(cmd: DomainCommand) -> Self {
        cmd.into_control_command()
    }
}

/// Structured report of runtime garbage collection and memory reclamation state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GarbageCollectionReport {
    pub dev_mode: bool,
    pub aggressive: bool,
    pub gc_interval_secs: u64,
    pub collections_count: u64,
    pub last_reclaimed_kb: u64,
    pub resource_cache_items: usize,
    pub resource_cache_capacity: usize,
    pub memory_used_mb: f64,
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

/// Full descriptor for a running widget returned by `ListWidgets`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetDescriptor {
    pub id: String,
    pub name: String,
    pub state: String,       // "running" | "disabled" | "loaded"
    pub opacity: f32,
    pub scale: f32,
    pub locked: bool,
    pub enabled: bool,
    pub quick_swap: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub manifest_path: String,
}

impl WidgetDescriptor {
    pub fn from_id(id: &str) -> Self {
        let name = id.split('.').last().unwrap_or(id)
            .replace('_', " ")
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        Self {
            id: id.to_string(),
            name,
            state: "running".to_string(),
            opacity: 1.0,
            scale: 1.0,
            locked: false,
            enabled: true,
            quick_swap: false,
            position_x: 100,
            position_y: 100,
            manifest_path: String::new(),
        }
    }
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
            open_apps_count: 0,
            browser_tabs_count: 0,
            audio_playing_apps_count: 0,
            gaming_apps_count: 0,
            dev_suite_apps_count: 0,
            other_apps_count: 0,
            master_volume_pct: 0.0,
            is_muted: false,
            battery_charge_pct: 0.0,
            battery_remaining_secs: 0,
            is_charging: false,
            total_gpu_count: 0,
            integrated_gpu_count: 0,
            dedicated_gpu_count: 0,
            total_display_count: 0,
            external_display_count: 0,
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
        assert_eq!(metric.open_apps_count, 0);
        assert_eq!(metric.master_volume_pct, 0.0);
        assert_eq!(metric.battery_charge_pct, 0.0);
    }

    #[test]
    fn test_control_command_domain_classification() {
        assert_eq!(ControlCommand::Ping.domain(), CommandDomain::Core);
        assert_eq!(
            ControlCommand::LoadWidget {
                manifest_path: "w.toml".into()
            }
            .domain(),
            CommandDomain::Widget
        );
        assert_eq!(
            ControlCommand::SetThemeMode {
                mode: "dark".into()
            }
            .domain(),
            CommandDomain::Theme
        );
        assert_eq!(
            ControlCommand::GetDiagnostics.domain(),
            CommandDomain::Diagnostics
        );
        assert_eq!(
            ControlCommand::ListSnapshots.domain(),
            CommandDomain::Snapshot
        );
        assert_eq!(
            ControlCommand::GetAuditLogs.domain(),
            CommandDomain::Enterprise
        );
        assert_eq!(
            ControlCommand::GetGarbageCollectionStats.domain(),
            CommandDomain::GarbageCollection
        );
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
            category: Some("all".to_string()),
        };
        let json_search = serde_json::to_string(&search_cmd).unwrap();
        let decoded_search: ControlCommand = serde_json::from_str(&json_search).unwrap();
        assert_eq!(decoded_search, search_cmd);

        // Verify backwards compatibility with legacy payloads omitting category
        let legacy_json = r#"{"SearchMarketplace":{"query":"monitoring"}}"#;
        let legacy_decoded: ControlCommand = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(
            legacy_decoded,
            ControlCommand::SearchMarketplace {
                query: "monitoring".to_string(),
                category: None,
            }
        );

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

    #[test]
    fn test_garbage_collection_control_commands_serialization() {
        let trigger_cmd = ControlCommand::TriggerGarbageCollection {
            aggressive: true,
            trim_working_set: true,
        };
        let json_trig = serde_json::to_string(&trigger_cmd).unwrap();
        let decoded_trig: ControlCommand = serde_json::from_str(&json_trig).unwrap();
        assert_eq!(decoded_trig, trigger_cmd);

        let policy_cmd = ControlCommand::SetGarbageCollectionPolicy {
            dev_mode: true,
            aggressive: false,
            interval_secs: 15,
        };
        let json_policy = serde_json::to_string(&policy_cmd).unwrap();
        let decoded_policy: ControlCommand = serde_json::from_str(&json_policy).unwrap();
        assert_eq!(decoded_policy, policy_cmd);

        let stats_cmd = ControlCommand::GetGarbageCollectionStats;
        let json_stats = serde_json::to_string(&stats_cmd).unwrap();
        let decoded_stats: ControlCommand = serde_json::from_str(&json_stats).unwrap();
        assert_eq!(decoded_stats, stats_cmd);

        let report = GarbageCollectionReport {
            dev_mode: true,
            aggressive: true,
            gc_interval_secs: 15,
            collections_count: 42,
            last_reclaimed_kb: 10240,
            resource_cache_items: 5,
            resource_cache_capacity: 50,
            memory_used_mb: 32.5,
        };
        let json_rep = serde_json::to_string(&report).unwrap();
        let decoded_rep: GarbageCollectionReport = serde_json::from_str(&json_rep).unwrap();
        assert_eq!(decoded_rep, report);
    }

    #[test]
    fn test_domain_command_sub_enums_roundtrip_all_domains() {
        let samples = vec![
            ControlCommand::Ping,
            ControlCommand::SetAdaptiveTickMode { enabled: true },
            ControlCommand::LoadWidget {
                manifest_path: "crates/perf_monitor_widget/widget.toml".to_string(),
            },
            ControlCommand::SetThemeMode {
                mode: "dark".to_string(),
            },
            ControlCommand::GetDiagnostics,
            ControlCommand::CreateSnapshot {
                name: "Pre-Upgrade".to_string(),
            },
            ControlCommand::RequestCapabilityToken {
                widget_id: "net_monitor".to_string(),
                capability: "network.http".to_string(),
            },
            ControlCommand::GetPrometheusMetrics,
            ControlCommand::SearchMarketplace {
                query: "audio".to_string(),
                category: Some("media".to_string()),
            },
            ControlCommand::SynthesizeWidget {
                prompt: "CPU core heatmap".to_string(),
            },
            ControlCommand::VerifyAuditChain,
            ControlCommand::SetDesktopProfile {
                profile_id: "Gaming".to_string(),
            },
            ControlCommand::TriggerGarbageCollection {
                aggressive: true,
                trim_working_set: true,
            },
        ];

        for cmd in samples {
            let domain_cmd = cmd.into_domain_command();
            assert_eq!(domain_cmd.domain(), cmd.domain());
            let roundtripped = domain_cmd.into_control_command();
            assert_eq!(roundtripped, cmd);
        }
    }
}


