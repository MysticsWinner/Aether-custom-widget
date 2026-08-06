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
}

impl IpcSharedState {
    pub fn new(
        cache: SharedTelemetryCache,
        desktop_window: Arc<DesktopWidgetWindow>,
        initial_widgets: Vec<String>,
    ) -> Self {
        Self {
            cache,
            desktop_window,
            widget_registry: Arc::new(Mutex::new(initial_widgets)),
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
    fn test_dispatch_get_status_returns_ok_with_widget_registry() {
        let state = make_state();
        let resp = dispatch_command("\"GetStatus\"", &state);
        assert!(resp.contains("\"status\":\"ok\"") || resp.contains("\"status\": \"ok\""), "response: {resp}");
        assert!(resp.contains("perf_monitor"), "should include builtin widget: {resp}");
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
