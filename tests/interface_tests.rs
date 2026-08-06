//! Interface Tests for the Aether Platform
//!
//! Validates the serialization format and IPC message layout protocols,
//! error handling boundaries for malformed payloads, and the API schemas.

use std::sync::{Arc, Mutex};

use core_engine::ipc_server::{dispatch_command, IpcSharedState};
use core_engine::DesktopWidgetWindow;
use ipc_protocol::ControlCommand;
use system_providers::SharedTelemetryCache;

fn create_test_shared_state() -> IpcSharedState {
    let cache = SharedTelemetryCache::default();
    let desktop_window = Arc::new(DesktopWidgetWindow::new());
    IpcSharedState::new(
        cache,
        desktop_window,
        vec!["aether.builtin.perf_monitor".to_string()],
    )
}

#[test]
fn test_interface_control_command_compatibility() {
    // 1. GetStatus JSON Compatibility check
    let get_status = ControlCommand::GetStatus;
    let json_get_status = serde_json::to_string(&get_status).unwrap();
    assert_eq!(json_get_status, "\"GetStatus\"");

    // 2. SetThemeMode JSON Compatibility check
    let theme_mode = ControlCommand::SetThemeMode {
        mode: "system".to_string(),
    };
    let json_theme = serde_json::to_string(&theme_mode).unwrap();
    assert!(json_theme.contains("\"SetThemeMode\""));
    assert!(json_theme.contains("\"mode\":\"system\"") || json_theme.contains("\"mode\": \"system\""));

    // 3. SetWidgetPosition JSON Compatibility check
    let position = ControlCommand::SetWidgetPosition {
        widget_id: "aether.custom.clock".to_string(),
        x: 100,
        y: 200,
    };
    let json_pos = serde_json::to_string(&position).unwrap();
    assert!(json_pos.contains("\"SetWidgetPosition\""));
    assert!(json_pos.contains("clock"));
}

#[test]
fn test_interface_edge_cases_and_error_handling() {
    let state = create_test_shared_state();

    // Case A: Completely malformed JSON payload
    let malformed_raw = "{invalid_json_payload";
    let response_malformed = dispatch_command(malformed_raw, &state);

    assert!(response_malformed.contains("\"status\":\"error\"") || response_malformed.contains("\"status\": \"error\""));
    assert!(response_malformed.contains("message"));

    // Case B: Unknown command structure
    let unknown_cmd = "\"NonExistentCommandName\"";
    let response_unknown = dispatch_command(unknown_cmd, &state);

    assert!(response_unknown.contains("\"status\":\"error\"") || response_unknown.contains("\"status\": \"error\""));
}

#[test]
fn test_interface_subsystem_health_report() {
    let state = create_test_shared_state();

    // Query health of subsystems
    let raw_command = "\"GetSubsystemHealth\"";
    let json_response = dispatch_command(raw_command, &state);

    // Verify all 9 subsystems are accounted for in the health report interface
    assert!(json_response.contains("telemetry_subsystem"));
    assert!(json_response.contains("gpu_render_engine"));
    assert!(json_response.contains("theme_engine"));
    assert!(json_response.contains("plugin_sandbox"));
    assert!(json_response.contains("profiler"));
    assert!(json_response.contains("marketplace"));
    assert!(json_response.contains("cloud_sync"));
    assert!(json_response.contains("ai_intelligence"));
    assert!(json_response.contains("production_readiness"));
    assert!(json_response.contains("Healthy"));
}
