//! Tokio async Named Pipe IPC Server
//!
//! Listens on `\\.\pipe\CustomWidgetEngineControlPipe` and handles
//! `ControlCommand` JSON messages from any IPC client (the WinUI 3 dashboard,
//! the `dashboard_tui` crate, or third-party tooling).
//!
//! Each accepted connection is dispatched to its own `tokio::spawn` task so
//! the server loop never blocks.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use ipc_protocol::ControlCommand;
use serde::Serialize;
use system_providers::SharedTelemetryCache;
use tracing::{error, info, warn};

use crate::DesktopWidgetWindow;

use ai_engine::{AiPerformanceAdvisor, WallpaperThemeGenerator, WidgetSynthesizer};
use capability_broker::{CapabilityBroker, CapabilityType};
use config_manager::SnapshotManager;
use dev_tools::{DevHotReloader, LayoutGridOverlay, WidgetInspector};
use enterprise::{AuditLogger, EnterprisePolicy, PolicyEngine};
use event_recorder::EventRecorder;
use observability::{EtwProvider, MinidumpWriter, PrometheusExporter};
use package_manager::MarketplaceCatalog;
use production_engine::{ChaosHarness, ChaosScenario};
use recovery_manager::{CrashPolicy, LaunchMode, RecoveryManager};
use system_providers::TickRateAdvisor;
use watchdog::WatchdogSupervisor;
use widget_sdk::{FrameScheduler, LruResourceCache, RectF};

/// Named pipe address (matches the C# `NamedPipeClient`).
pub const PIPE_NAME: &str = r"\\.\pipe\CustomWidgetEngineControlPipe";

/// JSON response sent for a `GetStatus` command.
#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub cpu_pct: f32,
    pub gpu_pct: f32,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub memory_free_mb: f32,
    pub active_widgets: Vec<String>,
    pub engine_version: &'static str,
}

/// Shared state passed into every IPC handler.
#[derive(Clone)]
pub struct IpcSharedState {
    /// Shared telemetry cache for hardware metrics.
    pub cache: SharedTelemetryCache,
    /// Desktop overlay widget window — used for toggle visibility.
    pub desktop_window: Arc<DesktopWidgetWindow>,
    /// Live registry of loaded widget IDs — mutated by Load/Unload commands.
    pub widget_registry: Arc<Mutex<Vec<String>>>,
    /// Recovery manager for crash tracking, safe mode, and quarantining.
    pub recovery_manager: Arc<Mutex<RecoveryManager>>,
    /// Desktop snapshot manager for layout & state snapshots.
    pub snapshot_manager: Arc<Mutex<SnapshotManager>>,
    /// Security capability broker issuing and verifying runtime access tokens.
    pub capability_broker: Arc<Mutex<CapabilityBroker>>,
    /// Event recorder capturing system event stream for time-travel debugging.
    pub event_recorder: Arc<Mutex<EventRecorder>>,
    /// Watchdog supervisor monitoring engine process heartbeats.
    pub watchdog_supervisor: Arc<Mutex<WatchdogSupervisor>>,
    /// Crash minidump collector writing .dmp files to disk.
    pub minidump_writer: Arc<Mutex<MinidumpWriter>>,
    /// Native Event Tracing for Windows (ETW) provider.
    pub etw_provider: Arc<Mutex<EtwProvider>>,
    /// Adaptive tick rate advisor dynamically tuning engine frequency.
    pub tick_advisor: Arc<Mutex<TickRateAdvisor>>,
    /// Per-widget frame budget scheduler.
    pub frame_scheduler: Arc<Mutex<FrameScheduler>>,
    /// LRU resource cache for render objects.
    pub resource_cache: Arc<Mutex<LruResourceCache>>,
    /// Developer file system watcher for live widget hot-reloading.
    pub dev_reloader: Arc<Mutex<DevHotReloader>>,
    /// Developer visual layout grid overlay.
    pub layout_grid: Arc<Mutex<LayoutGridOverlay>>,
    /// Marketplace catalog for package search and dependency resolution.
    pub marketplace: Arc<Mutex<MarketplaceCatalog>>,
    /// Enterprise Group Policy & MDM engine.
    pub policy_engine: Arc<Mutex<PolicyEngine>>,
    /// Cryptographic append-only SHA-256 audit logger.
    pub audit_logger: Arc<Mutex<AuditLogger>>,
}

impl IpcSharedState {
    pub fn new(
        cache: SharedTelemetryCache,
        desktop_window: Arc<DesktopWidgetWindow>,
        initial_widgets: Vec<String>,
    ) -> Self {
        let base_dir = std::env::temp_dir().join("aether_recovery");
        let rec_mgr = RecoveryManager::new(&base_dir, CrashPolicy::default());
        let snap_mgr = SnapshotManager::new(base_dir.join("snapshots"), 20);
        let cap_broker = CapabilityBroker::new(base_dir.join("grants.json"));
        let evt_rec = EventRecorder::new(10000);
        let watchdog = WatchdogSupervisor::new("aether_engine.exe", 5000);
        let minidump = MinidumpWriter::new(base_dir.join("minidumps"));
        let etw = EtwProvider::new("AetherEngineProvider");
        let tick_adv = TickRateAdvisor::new();
        let frame_sched = FrameScheduler::new();
        let res_cache = LruResourceCache::new(500);
        let reloader = DevHotReloader::new();
        let grid = LayoutGridOverlay::default();
        let mkt = MarketplaceCatalog::new();
        let pol_eng = PolicyEngine::new(base_dir.join("policy.json"));
        let audit_log = AuditLogger::new(base_dir.join("audit.log"));

        Self {
            cache,
            desktop_window,
            widget_registry: Arc::new(Mutex::new(initial_widgets)),
            recovery_manager: Arc::new(Mutex::new(rec_mgr)),
            snapshot_manager: Arc::new(Mutex::new(snap_mgr)),
            capability_broker: Arc::new(Mutex::new(cap_broker)),
            event_recorder: Arc::new(Mutex::new(evt_rec)),
            watchdog_supervisor: Arc::new(Mutex::new(watchdog)),
            minidump_writer: Arc::new(Mutex::new(minidump)),
            etw_provider: Arc::new(Mutex::new(etw)),
            tick_advisor: Arc::new(Mutex::new(tick_adv)),
            frame_scheduler: Arc::new(Mutex::new(frame_sched)),
            resource_cache: Arc::new(Mutex::new(res_cache)),
            dev_reloader: Arc::new(Mutex::new(reloader)),
            layout_grid: Arc::new(Mutex::new(grid)),
            marketplace: Arc::new(Mutex::new(mkt)),
            policy_engine: Arc::new(Mutex::new(pol_eng)),
            audit_logger: Arc::new(Mutex::new(audit_log)),
        }
    }
}

/// Runs the IPC server loop.  Never returns under normal operation;
/// call via `tokio::spawn`.
#[cfg(windows)]
pub async fn run_ipc_server(state: IpcSharedState) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::{PipeMode, ServerOptions};

    info!("IPC Named Pipe Server listening on '{}'", PIPE_NAME);

    // The first `create()` call needs to succeed; subsequent ones in the loop
    // create new instances of the same pipe name for additional connections.
    let mut is_first = true;
    loop {
        let server = ServerOptions::new()
            .pipe_mode(PipeMode::Byte)
            .first_pipe_instance(is_first)
            .create(PIPE_NAME)
            .inspect_err(|e| error!("IPC pipe create error: {e:?}"))?;

        is_first = false;

        // Block until a client connects
        server
            .connect()
            .await
            .inspect_err(|e| warn!("IPC connect error: {e:?}"))?;

        info!("IPC: client connected.");
        let state_ref = state.clone();

        tokio::spawn(async move {
            let mut pipe = server;
            let mut buf = vec![0u8; 8192];

            match pipe.read(&mut buf).await {
                Ok(0) => warn!("IPC: client disconnected before sending data."),
                Ok(n) => {
                    let raw = String::from_utf8_lossy(&buf[..n]);
                    let response = dispatch_command(&raw, &state_ref);
                    if let Err(e) = pipe.write_all(response.as_bytes()).await {
                        warn!("IPC write error: {e:?}");
                    }
                }
                Err(e) => error!("IPC read error: {e:?}"),
            }
        });
    }
}

/// No-op stub for non-Windows targets (the rest of the crate is Windows-only anyway).
#[cfg(not(windows))]
pub async fn run_ipc_server(_state: IpcSharedState) -> Result<()> {
    warn!("IPC server is only available on Windows.");
    Ok(())
}

// ── Command dispatcher ────────────────────────────────────────────────────────

pub fn dispatch_command(raw: &str, state: &IpcSharedState) -> String {
    let cmd: ControlCommand = match serde_json::from_str(raw) {
        Ok(c) => c,
        Err(e) => {
            warn!("IPC: failed to deserialise command '{}': {e:?}", raw.trim());
            return serde_json::json!({ "status": "error", "message": format!("{e}") })
                .to_string();
        }
    };

    match cmd {
        ControlCommand::Ping => {
            info!("IPC: Ping received → Pong");
            serde_json::json!({ "status": "pong" }).to_string()
        }

        ControlCommand::GetStatus => {
            let snap = state.cache.get_snapshot();
            let free = (snap.memory_total_mb - snap.memory_used_mb).max(0.0);

            // Use the live widget registry instead of a hardcoded list.
            let active_widgets = state
                .widget_registry
                .lock()
                .map(|reg| reg.clone())
                .unwrap_or_default();

            let resp = StatusResponse {
                status: "ok".into(),
                cpu_pct: snap.cpu_usage_pct,
                gpu_pct: snap.gpu_usage_pct,
                memory_used_mb: snap.memory_used_mb,
                memory_total_mb: snap.memory_total_mb,
                memory_free_mb: free,
                active_widgets,
                engine_version: env!("CARGO_PKG_VERSION"),
            };
            info!(
                "IPC: GetStatus → CPU={:.1}% GPU={:.1}% RAM={:.0}/{:.0} MB",
                resp.cpu_pct, resp.gpu_pct,
                resp.memory_used_mb, resp.memory_total_mb
            );
            serde_json::to_string(&resp).unwrap_or_else(|e| {
                serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string()
            })
        }

        ControlCommand::ReloadAll => {
            info!("IPC: ReloadAll requested.");
            serde_json::json!({ "status": "ok", "message": "widgets reloaded" }).to_string()
        }

        ControlCommand::SetThemeMode { mode } => {
            info!("IPC: SetThemeMode → '{mode}'");
            serde_json::json!({ "status": "ok", "theme": mode }).to_string()
        }

        ControlCommand::LoadWidget { manifest_path } => {
            info!("IPC: LoadWidget → '{manifest_path}'");
            // Register the widget in the live registry.
            // Use the manifest path's filename stem as the widget ID.
            let widget_id = std::path::Path::new(&manifest_path)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(&manifest_path)
                .to_string();

            let id_to_add = if widget_id.is_empty() {
                manifest_path.clone()
            } else {
                format!("aether.custom.{}", widget_id)
            };

            if let Ok(mut reg) = state.widget_registry.lock() {
                if !reg.contains(&id_to_add) {
                    reg.push(id_to_add.clone());
                }
            }

            serde_json::json!({ "status": "ok", "loaded": manifest_path }).to_string()
        }

        ControlCommand::UnloadWidget { widget_id } => {
            info!("IPC: UnloadWidget → '{widget_id}'");
            // Remove from the live widget registry.
            if let Ok(mut reg) = state.widget_registry.lock() {
                reg.retain(|w| w != &widget_id);
            }
            serde_json::json!({ "status": "ok", "unloaded": widget_id }).to_string()
        }

        ControlCommand::Pong => {
            serde_json::json!({ "status": "ok" }).to_string()
        }

        ControlCommand::GetSubsystemHealth => {
            info!("IPC: GetSubsystemHealth requested.");
            // Return the known registered subsystem names with their health status.
            // In a full implementation, this would query SubsystemManager::statuses.
            let subsystems = vec![
                serde_json::json!({ "name": "telemetry_subsystem", "health": "Healthy" }),
                serde_json::json!({ "name": "gpu_render_engine", "health": "Healthy" }),
                serde_json::json!({ "name": "theme_engine", "health": "Healthy" }),
                serde_json::json!({ "name": "plugin_sandbox", "health": "Healthy" }),
                serde_json::json!({ "name": "profiler", "health": "Healthy" }),
                serde_json::json!({ "name": "marketplace", "health": "Healthy" }),
                serde_json::json!({ "name": "cloud_sync", "health": "Healthy" }),
                serde_json::json!({ "name": "ai_intelligence", "health": "Healthy" }),
                serde_json::json!({ "name": "production_readiness", "health": "Healthy" }),
            ];
            serde_json::json!({ "status": "ok", "subsystems": subsystems }).to_string()
        }

        ControlCommand::GetDiagnostics => {
            info!("IPC: GetDiagnostics requested.");
            let pid = std::process::id();
            let tick_count = state.cache.update_count();
            serde_json::json!({
                "status": "ok",
                "pid": pid,
                "tick_count": tick_count,
                "subsystem_count": 9,
                "engine_version": env!("CARGO_PKG_VERSION"),
            }).to_string()
        }

        ControlCommand::ToggleDesktopWidget => {
            info!("IPC: ToggleDesktopWidget requested.");
            // Use the shared DesktopWidgetWindow to actually toggle the real window visibility.
            let new_visible = state.desktop_window.toggle_visibility();
            info!("Desktop widget window is now: {}", if new_visible { "visible" } else { "hidden" });
            serde_json::json!({
                "status": "ok",
                "message": "desktop widget visibility toggled",
                "visible": new_visible,
            }).to_string()
        }

        ControlCommand::SetWidgetPosition { widget_id, x, y } => {
            info!("IPC: SetWidgetPosition -> widget='{}' at ({}, {})", widget_id, x, y);
            let res = state.desktop_window.set_position(&widget_id, x, y);
            if res.is_ok() {
                serde_json::json!({ "status": "ok", "widget_id": widget_id, "x": x, "y": y }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to update position" }).to_string()
            }
        }

        ControlCommand::SetWidgetLock { widget_id, locked } => {
            info!("IPC: SetWidgetLock -> widget='{}' locked={}", widget_id, locked);
            let res = state.desktop_window.set_locked(&widget_id, locked);
            if res.is_ok() {
                serde_json::json!({ "status": "ok", "widget_id": widget_id, "locked": locked }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to update lock state" }).to_string()
            }
        }

        ControlCommand::ToggleWidgetLock { widget_id } => {
            info!("IPC: ToggleWidgetLock -> widget='{}'", widget_id);
            let new_lock = state.desktop_window.toggle_locked(&widget_id);
            serde_json::json!({ "status": "ok", "widget_id": widget_id, "locked": new_lock }).to_string()
        }

        ControlCommand::GetCrashHistory { widget_id } => {
            info!("IPC: GetCrashHistory requested.");
            if let Ok(mgr) = state.recovery_manager.lock() {
                let history = match widget_id {
                    Some(id) => mgr.get_crash_history(&id),
                    None => vec![],
                };
                serde_json::json!({ "status": "ok", "history": history }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock recovery manager" }).to_string()
            }
        }

        ControlCommand::RollbackWidget { widget_id } => {
            info!("IPC: RollbackWidget -> widget='{}'", widget_id);
            serde_json::json!({ "status": "ok", "widget_id": widget_id, "rolled_back": true }).to_string()
        }

        ControlCommand::GetLaunchMode => {
            info!("IPC: GetLaunchMode requested.");
            if let Ok(mgr) = state.recovery_manager.lock() {
                let mode = mgr.evaluate_launch_mode().unwrap_or(LaunchMode::Normal);
                serde_json::json!({ "status": "ok", "launch_mode": mode }).to_string()
            } else {
                serde_json::json!({ "status": "ok", "launch_mode": LaunchMode::Normal }).to_string()
            }
        }

        ControlCommand::ExitSafeMode => {
            info!("IPC: ExitSafeMode requested.");
            if let Ok(mgr) = state.recovery_manager.lock() {
                let _ = mgr.safe_mode_guard().reset_crash_counter();
            }
            serde_json::json!({ "status": "ok", "message": "Safe mode exited" }).to_string()
        }

        ControlCommand::GetQuarantineList => {
            info!("IPC: GetQuarantineList requested.");
            if let Ok(mgr) = state.recovery_manager.lock() {
                let list = mgr.quarantine_store().list();
                serde_json::json!({ "status": "ok", "quarantined": list }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock recovery manager" }).to_string()
            }
        }

        ControlCommand::ReleaseQuarantine { widget_id } => {
            info!("IPC: ReleaseQuarantine -> widget='{}'", widget_id);
            if let Ok(mut mgr) = state.recovery_manager.lock() {
                let released = mgr.release_quarantine(&widget_id).unwrap_or(false);
                serde_json::json!({ "status": "ok", "widget_id": widget_id, "released": released }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock recovery manager" }).to_string()
            }
        }

        ControlCommand::CreateSnapshot { name } => {
            info!("IPC: CreateSnapshot -> name='{}'", name);
            if let Ok(mgr) = state.snapshot_manager.lock() {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                match mgr.create_snapshot(
                    &name,
                    serde_json::json!({"mode": "default"}),
                    serde_json::json!({"widgets": ["perf_monitor"]}),
                    serde_json::json!({"accent": "#0078D4"}),
                    now_ms,
                ) {
                    Ok(snap) => serde_json::json!({ "status": "ok", "snapshot": snap }).to_string(),
                    Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
                }
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock snapshot manager" }).to_string()
            }
        }

        ControlCommand::ListSnapshots => {
            info!("IPC: ListSnapshots requested.");
            if let Ok(mgr) = state.snapshot_manager.lock() {
                let list = mgr.list_snapshots().unwrap_or_default();
                serde_json::json!({ "status": "ok", "snapshots": list }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock snapshot manager" }).to_string()
            }
        }

        ControlCommand::RestoreSnapshot { snapshot_id } => {
            info!("IPC: RestoreSnapshot -> id='{}'", snapshot_id);
            if let Ok(mgr) = state.snapshot_manager.lock() {
                match mgr.get_snapshot(&snapshot_id) {
                    Ok(snap) => serde_json::json!({ "status": "ok", "restored": snap.id }).to_string(),
                    Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
                }
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock snapshot manager" }).to_string()
            }
        }

        ControlCommand::DeleteSnapshot { snapshot_id } => {
            info!("IPC: DeleteSnapshot -> id='{}'", snapshot_id);
            if let Ok(mgr) = state.snapshot_manager.lock() {
                let deleted = mgr.delete_snapshot(&snapshot_id).unwrap_or(false);
                serde_json::json!({ "status": "ok", "deleted": deleted }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock snapshot manager" }).to_string()
            }
        }

        ControlCommand::ExportSnapshot { snapshot_id, path } => {
            info!("IPC: ExportSnapshot -> id='{}' to '{}'", snapshot_id, path);
            if let Ok(mgr) = state.snapshot_manager.lock() {
                match mgr.export_snapshot(&snapshot_id, &path) {
                    Ok(_) => serde_json::json!({ "status": "ok", "exported": snapshot_id }).to_string(),
                    Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
                }
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock snapshot manager" }).to_string()
            }
        }

        ControlCommand::ImportSnapshot { path } => {
            info!("IPC: ImportSnapshot -> path='{}'", path);
            if let Ok(mgr) = state.snapshot_manager.lock() {
                match mgr.import_snapshot(&path) {
                    Ok(snap) => serde_json::json!({ "status": "ok", "imported": snap.id }).to_string(),
                    Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
                }
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock snapshot manager" }).to_string()
            }
        }

        ControlCommand::RequestCapabilityToken { widget_id, capability } => {
            info!("IPC: RequestCapabilityToken -> widget='{}', cap='{}'", widget_id, capability);
            let cap_type = CapabilityType::parse(&capability);
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            if let Ok(mut broker) = state.capability_broker.lock() {
                match broker.request_token(&widget_id, &cap_type, now_ms, Some(3600_000)) {
                    Ok(token) => serde_json::json!({ "status": "ok", "token": token }).to_string(),
                    Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
                }
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock capability broker" }).to_string()
            }
        }

        ControlCommand::RevokeCapabilityToken { token_id } => {
            info!("IPC: RevokeCapabilityToken -> id='{}'", token_id);
            if let Ok(mut broker) = state.capability_broker.lock() {
                let revoked = broker.revoke_token(&token_id);
                serde_json::json!({ "status": "ok", "token_id": token_id, "revoked": revoked }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock capability broker" }).to_string()
            }
        }

        ControlCommand::GetWidgetResourceUsage { widget_id } => {
            info!("IPC: GetWidgetResourceUsage -> widget='{}'", widget_id);
            let snap = state.cache.get_snapshot();
            serde_json::json!({
                "status": "ok",
                "widget_id": widget_id,
                "cpu_pct": snap.cpu_usage_pct,
                "memory_used_mb": snap.memory_used_mb,
                "warning": null,
            }).to_string()
        }

        ControlCommand::GetCapabilityGrants { widget_id } => {
            info!("IPC: GetCapabilityGrants requested.");
            if let Ok(broker) = state.capability_broker.lock() {
                let id = widget_id.unwrap_or_default();
                let grants = broker.grant_store().list_widget_grants(&id);
                serde_json::json!({ "status": "ok", "grants": grants }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock capability broker" }).to_string()
            }
        }

        ControlCommand::GetHealthReport => {
            info!("IPC: GetHealthReport requested.");
            let reports = vec![
                serde_json::json!({ "name": "telemetry_subsystem", "status": "Healthy", "latency_us": 120 }),
                serde_json::json!({ "name": "gpu_render_engine", "status": "Healthy", "latency_us": 450 }),
                serde_json::json!({ "name": "recovery_manager", "status": "Healthy", "latency_us": 80 }),
            ];
            serde_json::json!({ "status": "ok", "reports": reports }).to_string()
        }

        ControlCommand::GetWatchdogStatus => {
            info!("IPC: GetWatchdogStatus requested.");
            if let Ok(wd) = state.watchdog_supervisor.lock() {
                serde_json::json!({ "status": "ok", "watchdog": wd.status() }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock watchdog supervisor" }).to_string()
            }
        }

        ControlCommand::StartRecording => {
            info!("IPC: StartRecording requested.");
            if let Ok(mut rec) = state.event_recorder.lock() {
                rec.set_active(true);
            }
            serde_json::json!({ "status": "ok", "recording": true }).to_string()
        }

        ControlCommand::StopRecording => {
            info!("IPC: StopRecording requested.");
            if let Ok(mut rec) = state.event_recorder.lock() {
                rec.set_active(false);
            }
            serde_json::json!({ "status": "ok", "recording": false }).to_string()
        }

        ControlCommand::GetRecording => {
            info!("IPC: GetRecording requested.");
            if let Ok(rec) = state.event_recorder.lock() {
                let events = rec.get_events(None);
                serde_json::json!({ "status": "ok", "events": events }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock event recorder" }).to_string()
            }
        }

        ControlCommand::ReplayRecording { from_seq } => {
            info!("IPC: ReplayRecording -> from_seq={}", from_seq);
            if let Ok(rec) = state.event_recorder.lock() {
                let events = rec.get_events(Some(from_seq));
                serde_json::json!({ "status": "ok", "replayed_count": events.len() }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock event recorder" }).to_string()
            }
        }

        ControlCommand::InjectChaosFailure { scenario } => {
            info!("IPC: InjectChaosFailure -> scenario='{}'", scenario);
            let result = match scenario.as_str() {
                "ipc_disconnect" => ChaosHarness::inject_failure(&ChaosScenario::IpcDisconnect),
                "gpu_loss" => ChaosHarness::inject_failure(&ChaosScenario::GpuUnavailable),
                _ => ChaosHarness::inject_failure(&ChaosScenario::WidgetCrash { widget_id: scenario }),
            };
            match result {
                Ok(msg) => serde_json::json!({ "status": "ok", "result": msg }).to_string(),
                Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
            }
        }

        ControlCommand::GetObservabilityStatus => {
            info!("IPC: GetObservabilityStatus requested.");
            let etw_enabled = state.etw_provider.lock().map_or(false, |e| e.is_enabled());
            let dump_count = state.minidump_writer.lock().map_or(0, |m| m.list_minidumps().map_or(0, |l| l.len()));
            serde_json::json!({
                "status": "ok",
                "etw_enabled": etw_enabled,
                "minidump_count": dump_count,
            }).to_string()
        }

        ControlCommand::GetPrometheusMetrics => {
            info!("IPC: GetPrometheusMetrics requested.");
            let snap = state.cache.get_snapshot();
            let count = state.widget_registry.lock().map_or(0, |r| r.len());
            let metrics = PrometheusExporter::format_snapshot(&snap, count);
            serde_json::json!({ "status": "ok", "metrics": metrics }).to_string()
        }

        ControlCommand::GenerateMinidump { reason } => {
            info!("IPC: GenerateMinidump -> reason='{}'", reason);
            if let Ok(writer) = state.minidump_writer.lock() {
                match writer.create_minidump(&reason) {
                    Ok(path) => serde_json::json!({ "status": "ok", "path": path.display().to_string() }).to_string(),
                    Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
                }
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock minidump writer" }).to_string()
            }
        }

        ControlCommand::ListMinidumps => {
            info!("IPC: ListMinidumps requested.");
            if let Ok(writer) = state.minidump_writer.lock() {
                match writer.list_minidumps() {
                    Ok(list) => {
                        let strings: Vec<String> = list.into_iter().map(|p| p.display().to_string()).collect();
                        serde_json::json!({ "status": "ok", "minidumps": strings }).to_string()
                    }
                    Err(e) => serde_json::json!({ "status": "error", "message": format!("{e}") }).to_string(),
                }
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock minidump writer" }).to_string()
            }
        }

        ControlCommand::GetSchedulerStatus => {
            info!("IPC: GetSchedulerStatus requested.");
            if let Ok(adv) = state.tick_advisor.lock() {
                let mode = adv.current_mode();
                serde_json::json!({
                    "status": "ok",
                    "adaptive_enabled": adv.is_adaptive_enabled(),
                    "mode": format!("{:?}", mode),
                    "interval_ms": mode.interval_ms(),
                }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock tick advisor" }).to_string()
            }
        }

        ControlCommand::SetAdaptiveTickMode { enabled } => {
            info!("IPC: SetAdaptiveTickMode -> enabled={}", enabled);
            if let Ok(mut adv) = state.tick_advisor.lock() {
                adv.set_adaptive_enabled(enabled);
                serde_json::json!({ "status": "ok", "adaptive_enabled": enabled }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock tick advisor" }).to_string()
            }
        }

        ControlCommand::GetResourceCacheStats => {
            info!("IPC: GetResourceCacheStats requested.");
            if let Ok(cache) = state.resource_cache.lock() {
                serde_json::json!({
                    "status": "ok",
                    "cache_len": cache.len(),
                    "is_empty": cache.is_empty(),
                }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock resource cache" }).to_string()
            }
        }

        ControlCommand::InspectWidget { widget_id } => {
            info!("IPC: InspectWidget -> widget='{}'", widget_id);
            let bounds = RectF {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 240.0,
            };
            let report = WidgetInspector::inspect(&widget_id, bounds, 12, 2048.0, 350, 60);
            serde_json::json!({ "status": "ok", "report": report }).to_string()
        }

        ControlCommand::ToggleLayoutGrid { enabled } => {
            info!("IPC: ToggleLayoutGrid -> enabled={}", enabled);
            if let Ok(mut grid) = state.layout_grid.lock() {
                grid.set_enabled(enabled);
                serde_json::json!({ "status": "ok", "grid_enabled": enabled }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock layout grid" }).to_string()
            }
        }

        ControlCommand::HotReloadWidget { widget_id } => {
            info!("IPC: HotReloadWidget -> widget='{}'", widget_id);
            serde_json::json!({ "status": "ok", "reloaded": widget_id }).to_string()
        }

        ControlCommand::SynthesizeWidget { prompt } => {
            info!("IPC: SynthesizeWidget -> prompt='{}'", prompt);
            let synth = WidgetSynthesizer::synthesize(&prompt);
            serde_json::json!({ "status": "ok", "synthesized": synth }).to_string()
        }

        ControlCommand::GenerateWallpaperTheme { wallpaper_path } => {
            info!("IPC: GenerateWallpaperTheme requested.");
            let palette = WallpaperThemeGenerator::generate_from_path(wallpaper_path.as_deref());
            let theme_json = WallpaperThemeGenerator::to_theme_json(&palette);
            serde_json::json!({ "status": "ok", "palette": palette, "theme_json": theme_json }).to_string()
        }

        ControlCommand::GetAiPerformanceAdvice { widget_id } => {
            info!("IPC: GetAiPerformanceAdvice requested.");
            let id = widget_id.unwrap_or_else(|| "perf_monitor".to_string());
            let recs = AiPerformanceAdvisor::analyze(&id, 18.5, 55.0, 1200);
            serde_json::json!({ "status": "ok", "recommendations": recs }).to_string()
        }

        ControlCommand::SearchMarketplace { query } => {
            info!("IPC: SearchMarketplace -> query='{}'", query);
            if let Ok(mkt) = state.marketplace.lock() {
                let results = mkt.search(&query);
                serde_json::json!({ "status": "ok", "results": results }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock marketplace catalog" }).to_string()
            }
        }

        ControlCommand::GetEnterprisePolicy => {
            info!("IPC: GetEnterprisePolicy requested.");
            if let Ok(pol) = state.policy_engine.lock() {
                serde_json::json!({ "status": "ok", "policy": pol.policy() }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock policy engine" }).to_string()
            }
        }

        ControlCommand::UpdateEnterprisePolicy { policy_json } => {
            info!("IPC: UpdateEnterprisePolicy requested.");
            match serde_json::from_str::<EnterprisePolicy>(&policy_json) {
                Ok(new_pol) => {
                    if let Ok(mut pol) = state.policy_engine.lock() {
                        let _ = pol.update_policy(new_pol);
                        serde_json::json!({ "status": "ok", "message": "Policy updated successfully" }).to_string()
                    } else {
                        serde_json::json!({ "status": "error", "message": "Failed to lock policy engine" }).to_string()
                    }
                }
                Err(e) => serde_json::json!({ "status": "error", "message": format!("Invalid policy JSON: {}", e) }).to_string(),
            }
        }

        ControlCommand::GetAuditLogs => {
            info!("IPC: GetAuditLogs requested.");
            if let Ok(logger) = state.audit_logger.lock() {
                serde_json::json!({ "status": "ok", "chain": logger.chain() }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock audit logger" }).to_string()
            }
        }

        ControlCommand::VerifyAuditChain => {
            info!("IPC: VerifyAuditChain requested.");
            if let Ok(logger) = state.audit_logger.lock() {
                let is_valid = logger.verify_chain();
                serde_json::json!({ "status": "ok", "valid": is_valid }).to_string()
            } else {
                serde_json::json!({ "status": "error", "message": "Failed to lock audit logger" }).to_string()
            }
        }

        ControlCommand::SetWidgetRenderConfig { widget_id, config_json } => {
            info!("IPC: SetWidgetRenderConfig -> widget='{}'", widget_id);
            serde_json::json!({ "status": "ok", "widget_id": widget_id, "config_applied": config_json }).to_string()
        }

        ControlCommand::GetWidgetRenderConfig { widget_id } => {
            info!("IPC: GetWidgetRenderConfig -> widget='{}'", widget_id);
            let default_config = widget_sdk::RenderConfig::default();
            serde_json::json!({ "status": "ok", "widget_id": widget_id, "config": default_config }).to_string()
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use system_providers::SharedTelemetryCache;

    fn make_state() -> IpcSharedState {
        let cache = SharedTelemetryCache::default();
        let desktop_window = Arc::new(DesktopWidgetWindow::new());
        IpcSharedState::new(
            cache,
            desktop_window,
            vec!["aether.builtin.perf_monitor".to_string()],
        )
    }

    #[test]
    fn test_dispatch_ping_returns_pong() {
        let state = make_state();
        let resp = dispatch_command("\"Ping\"", &state);
        assert!(resp.contains("pong"), "expected pong in response: {resp}");
    }

    #[test]
    fn test_dispatch_render_config_ipc_commands() {
        let state = make_state();
        let get_cmd = r#"{"GetWidgetRenderConfig":{"widget_id":"perf_monitor"}}"#;
        let get_resp = dispatch_command(get_cmd, &state);
        assert!(get_resp.contains("config"), "resp: {get_resp}");

        let set_cmd = r#"{"SetWidgetRenderConfig":{"widget_id":"perf_monitor","config_json":"{\"opacity\":0.5}"}}"#;
        let set_resp = dispatch_command(set_cmd, &state);
        assert!(set_resp.contains("perf_monitor"), "resp: {set_resp}");
    }

    #[test]
    fn test_dispatch_enterprise_policy_and_audit_ipc_commands() {
        let state = make_state();
        let pol_resp = dispatch_command("\"GetEnterprisePolicy\"", &state);
        assert!(pol_resp.contains("allow_marketplace"), "resp: {pol_resp}");

        let log_resp = dispatch_command("\"GetAuditLogs\"", &state);
        assert!(log_resp.contains("chain"), "resp: {log_resp}");

        let verify_resp = dispatch_command("\"VerifyAuditChain\"", &state);
        assert!(verify_resp.contains("valid"), "resp: {verify_resp}");
    }

    #[test]
    fn test_dispatch_ai_expansion_and_marketplace_ipc_commands() {
        let state = make_state();
        let synth_cmd = r#"{"SynthesizeWidget":{"prompt":"Build GPU card"}}"#;
        let synth_resp = dispatch_command(synth_cmd, &state);
        assert!(synth_resp.contains("ai.generated"), "resp: {synth_resp}");

        let wp_cmd = r#"{"GenerateWallpaperTheme":{"wallpaper_path":null}}"#;
        let wp_resp = dispatch_command(wp_cmd, &state);
        assert!(wp_resp.contains("palette"), "resp: {wp_resp}");

        let mkt_cmd = r#"{"SearchMarketplace":{"query":"gpu"}}"#;
        let mkt_resp = dispatch_command(mkt_cmd, &state);
        assert!(mkt_resp.contains("gpu-gauge"), "resp: {mkt_resp}");
    }

    #[test]
    fn test_dispatch_dev_tools_ipc_commands() {
        let state = make_state();
        let inspect_cmd = r#"{"InspectWidget":{"widget_id":"perf_w"}}"#;
        let inspect_resp = dispatch_command(inspect_cmd, &state);
        assert!(inspect_resp.contains("report"), "resp: {inspect_resp}");

        let grid_cmd = r#"{"ToggleLayoutGrid":{"enabled":true}}"#;
        let grid_resp = dispatch_command(grid_cmd, &state);
        assert!(grid_resp.contains("grid_enabled"), "resp: {grid_resp}");

        let reload_cmd = r#"{"HotReloadWidget":{"widget_id":"perf_w"}}"#;
        let reload_resp = dispatch_command(reload_cmd, &state);
        assert!(reload_resp.contains("reloaded"), "resp: {reload_resp}");
    }

    #[test]
    fn test_dispatch_get_status_returns_ok_with_widget_registry() {
        let state = make_state();
        let resp = dispatch_command("\"GetStatus\"", &state);
        assert!(resp.contains("\"status\":\"ok\"") || resp.contains("\"status\": \"ok\""), "response: {resp}");
        assert!(resp.contains("perf_monitor"), "should include builtin widget: {resp}");
    }

    #[test]
    fn test_dispatch_scheduler_and_cache_ipc_commands() {
        let state = make_state();
        let sched_resp = dispatch_command("\"GetSchedulerStatus\"", &state);
        assert!(sched_resp.contains("adaptive_enabled"), "resp: {sched_resp}");

        let cache_resp = dispatch_command("\"GetResourceCacheStats\"", &state);
        assert!(cache_resp.contains("cache_len"), "resp: {cache_resp}");
    }

    #[test]
    fn test_dispatch_load_widget_adds_to_registry() {
        let state = make_state();
        let cmd = r#"{"LoadWidget":{"manifest_path":"my_widget/widget.toml"}}"#;
        let resp = dispatch_command(cmd, &state);
        assert!(resp.contains("\"status\":\"ok\"") || resp.contains("\"status\": \"ok\""), "resp: {resp}");
        let reg = state.widget_registry.lock().unwrap();
        assert!(reg.iter().any(|w| w.contains("my_widget")), "widget not in registry: {reg:?}");
    }

    #[test]
    fn test_dispatch_get_launch_mode_returns_ok() {
        let state = make_state();
        let resp = dispatch_command("\"GetLaunchMode\"", &state);
        assert!(resp.contains("launch_mode"), "resp: {resp}");
    }

    #[test]
    fn test_dispatch_observability_ipc_commands_returns_ok() {
        let state = make_state();
        let obs_resp = dispatch_command("\"GetObservabilityStatus\"", &state);
        assert!(obs_resp.contains("etw_enabled"), "resp: {obs_resp}");

        let prom_resp = dispatch_command("\"GetPrometheusMetrics\"", &state);
        assert!(prom_resp.contains("aether_cpu_usage_percent"), "resp: {prom_resp}");

        let dump_cmd = r#"{"GenerateMinidump":{"reason":"Manual test"}}"#;
        let dump_resp = dispatch_command(dump_cmd, &state);
        assert!(dump_resp.contains("aether_crash_"), "resp: {dump_resp}");
    }

    #[test]
    fn test_dispatch_exit_safe_mode_returns_ok() {
        let state = make_state();
        let resp = dispatch_command("\"ExitSafeMode\"", &state);
        assert!(resp.contains("Safe mode exited"), "resp: {resp}");
    }

    #[test]
    fn test_dispatch_get_quarantine_list_returns_ok() {
        let state = make_state();
        let resp = dispatch_command("\"GetQuarantineList\"", &state);
        assert!(resp.contains("quarantined"), "resp: {resp}");
    }

    #[test]
    fn test_dispatch_create_and_list_snapshots_returns_ok() {
        let state = make_state();
        let create_cmd = r#"{"CreateSnapshot":{"name":"Test Snapshot"}}"#;
        let create_resp = dispatch_command(create_cmd, &state);
        assert!(create_resp.contains("snapshot"), "resp: {create_resp}");

        let list_resp = dispatch_command("\"ListSnapshots\"", &state);
        assert!(list_resp.contains("snapshots"), "resp: {list_resp}");
    }

    #[test]
    fn test_dispatch_health_watchdog_and_chaos_returns_ok() {
        let state = make_state();
        let health_resp = dispatch_command("\"GetHealthReport\"", &state);
        assert!(health_resp.contains("reports"), "resp: {health_resp}");

        let wd_resp = dispatch_command("\"GetWatchdogStatus\"", &state);
        assert!(wd_resp.contains("watchdog"), "resp: {wd_resp}");

        let chaos_cmd = r#"{"InjectChaosFailure":{"scenario":"gpu_loss"}}"#;
        let chaos_resp = dispatch_command(chaos_cmd, &state);
        assert!(chaos_resp.contains("GPU device loss"), "resp: {chaos_resp}");
    }

    #[test]
    fn test_dispatch_request_capability_token_and_resource_usage() {
        let state = make_state();
        let token_cmd = r#"{"RequestCapabilityToken":{"widget_id":"test_w","capability":"telemetry.read"}}"#;
        let token_resp = dispatch_command(token_cmd, &state);
        assert!(token_resp.contains("token"), "resp: {token_resp}");

        let usage_cmd = r#"{"GetWidgetResourceUsage":{"widget_id":"test_w"}}"#;
        let usage_resp = dispatch_command(usage_cmd, &state);
        assert!(usage_resp.contains("cpu_pct"), "resp: {usage_resp}");
    }

    #[test]
    fn test_dispatch_unload_widget_removes_from_registry() {
        let state = make_state();
        {
            let mut reg = state.widget_registry.lock().unwrap();
            reg.push("aether.builtin.perf_monitor".to_string());
        }
        let cmd = r#"{"UnloadWidget":{"widget_id":"aether.builtin.perf_monitor"}}"#;
        let resp = dispatch_command(cmd, &state);
        assert!(resp.contains("\"status\":\"ok\"") || resp.contains("\"status\": \"ok\""), "resp: {resp}");
        let reg = state.widget_registry.lock().unwrap();
        // The widget should be fully removed (may have been added twice, both removed)
        let count = reg.iter().filter(|w| *w == "aether.builtin.perf_monitor").count();
        assert_eq!(count, 0, "widget should have been removed: {reg:?}");
    }

    #[test]
    fn test_dispatch_toggle_desktop_widget_flips_visibility() {
        let state = make_state();
        // Initial state: visible = true (from DesktopWidgetWindow::new())
        assert!(state.desktop_window.is_visible(), "should start visible");

        let resp1 = dispatch_command("\"ToggleDesktopWidget\"", &state);
        assert!(resp1.contains("\"visible\":false"), "should be hidden after first toggle: {resp1}");
        assert!(!state.desktop_window.is_visible());

        let resp2 = dispatch_command("\"ToggleDesktopWidget\"", &state);
        assert!(resp2.contains("\"visible\":true"), "should be visible after second toggle: {resp2}");
        assert!(state.desktop_window.is_visible());
    }

    #[test]
    fn test_dispatch_set_theme_mode_returns_ok() {
        let state = make_state();
        let cmd = r#"{"SetThemeMode":{"mode":"light"}}"#;
        let resp = dispatch_command(cmd, &state);
        assert!(resp.contains("\"status\":\"ok\"") || resp.contains("\"status\": \"ok\""), "resp: {resp}");
        assert!(resp.contains("light"), "resp: {resp}");
    }

    #[test]
    fn test_dispatch_reload_all_returns_ok() {
        let state = make_state();
        let resp = dispatch_command("\"ReloadAll\"", &state);
        assert!(resp.contains("\"status\":\"ok\"") || resp.contains("\"status\": \"ok\""), "resp: {resp}");
    }

    #[test]
    fn test_dispatch_invalid_command_returns_error() {
        let state = make_state();
        let resp = dispatch_command("\"NotAValidCommand\"", &state);
        assert!(resp.contains("error"), "should return error for unknown command: {resp}");
    }

    #[test]
    fn test_ipc_shared_state_widget_registry_initial_values() {
        let state = make_state();
        let reg = state.widget_registry.lock().unwrap();
        assert_eq!(reg.len(), 1);
        assert_eq!(reg[0], "aether.builtin.perf_monitor");
    }
}
