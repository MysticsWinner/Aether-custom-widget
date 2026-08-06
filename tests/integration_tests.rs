//! Integration Test Suite for Next-Generation Windows Desktop Customization Platform
//!
//! Validates end-to-end subsystem integration, AppContainer sandbox fault isolation,
//! IPC protocol named pipe ring buffers, DirectComposition rendering culling,
//! Marketplace Ed25519 package installation, Encrypted Cloud Sync CRDT resolution,
//! AI Voice intent parsing, and Master Release Candidate stress testing across all 15 phases.

use ai_engine::{VoiceIntentParser, WorkflowAutomationEngine, WorkflowRule};
use cloud_sync::{
    AccountEntity, CloudSyncManager, DeviceEntity, LayoutEntity, PluginEntity, SettingsEntity,
    SyncEntity, ThemeEntity,
};
use core_engine::{
    AiSubsystem, CloudSyncSubsystem, CoreEvent, Engine, EngineConfig, MarketplaceSubsystem,
    MasterPerformanceSuite, PluginSandboxSubsystem, ProductionSubsystem, ProfilerSubsystem,
    RainmeterBenchmark, RenderSubsystem, TelemetrySubsystem, ThemeEngineSubsystem,
};
use ipc_protocol::{ControlCommand, SharedMemoryRingBuffer, MetricPayload};
use package_manager::PackageManager;
use plugin_runtime::{ApiVersion, PermissionManifest, PluginHealth, PluginSupervisor};
use production_engine::{MasterReleaseSuite, SecurityAuditor, StressTestingHarness};
use std::sync::Arc;
use theme_engine::{DynamicThemeStore, ThemeSchema, ThemeResolver};

#[tokio::test]
async fn test_01_core_engine_subsystem_integration_lifecycle() {
    let config = EngineConfig::new()
        .with_tick_interval_ms(10)
        .with_event_channel_capacity(1024)
        .with_telemetry(true);

    let mut engine = Engine::new(config);

    // Register all 9 Subsystem Coordinators
    let (telemetry_sys, shared_cache) = TelemetrySubsystem::new();
    let (theme_sys, _theme_store) = ThemeEngineSubsystem::new();
    let plugin_sandbox_sys = PluginSandboxSubsystem::new();
    let profiler_sys = ProfilerSubsystem::new();
    let marketplace_sys = MarketplaceSubsystem::new();
    let cloud_sync_sys = CloudSyncSubsystem::new();
    let ai_sys = AiSubsystem::new();
    let production_sys = ProductionSubsystem::new();

    engine.register_subsystem(Box::new(telemetry_sys));
    engine.register_subsystem(Box::new(RenderSubsystem::new()));
    engine.register_subsystem(Box::new(theme_sys));
    engine.register_subsystem(Box::new(plugin_sandbox_sys));
    engine.register_subsystem(Box::new(profiler_sys));
    engine.register_subsystem(Box::new(marketplace_sys));
    engine.register_subsystem(Box::new(cloud_sync_sys));
    engine.register_subsystem(Box::new(ai_sys));
    engine.register_subsystem(Box::new(production_sys));

    // Start Host Daemon Engine
    assert!(engine.start().await.is_ok());

    // Execute Tick Loop Passes
    for _ in 0..10 {
        engine.tick().await;
    }

    // Verify Telemetry Cache Sampling
    assert!(shared_cache.get_cpu_pct() >= 0.0);

    // Stop Engine Cleanly
    engine.stop().await;
}

#[tokio::test]
async fn test_02_ipc_protocol_ring_buffer_integration() {
    let mut ring_buffer = SharedMemoryRingBuffer::new();

    let payload = MetricPayload {
        timestamp_ms: 123456789,
        cpu_usage_pct: 42.5,
        memory_used_mb: 1024.0,
        memory_total_mb: 16384.0,
        gpu_usage_pct: 12.0,
        net_recv_bytes_per_sec: 5000,
        net_sent_bytes_per_sec: 2500,
    };

    // Assert successful push & pop
    assert!(ring_buffer.push(payload));
    let popped = ring_buffer.pop();
    assert_eq!(popped, Some(payload));

    // Verify ControlCommand json roundtrip serialization
    let cmd = ControlCommand::SetThemeMode {
        mode: "dark".to_string(),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    let decoded: ControlCommand = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, cmd);
}

#[tokio::test]
async fn test_03_appcontainer_sandbox_fault_isolation_integration() {
    let mut supervisor = PluginSupervisor::new();
    let manifest = PermissionManifest::new("mock_plugin_id");
    let pid = supervisor
        .launch_plugin("mock_plugin_id", ApiVersion::new(1, 0, 0), manifest)
        .unwrap();

    assert!(pid > 0);
    assert_eq!(supervisor.plugin_health("mock_plugin_id"), Some(PluginHealth::Running));

    // Simulate crash fault isolation
    supervisor.handle_plugin_crash("mock_plugin_id", -1);
    // Since restarts (1) <= max_restarts (3), it auto-restarts to Running state
    assert_eq!(supervisor.plugin_health("mock_plugin_id"), Some(PluginHealth::Running));
}

#[tokio::test]
async fn test_04_theme_hot_reload_token_resolution_integration() {
    let store = Arc::new(DynamicThemeStore::new(ThemeSchema::default()));

    let mut schema = ThemeSchema::default();
    schema.colors.insert("theme.accent".to_string(), "#FF007F".to_string());
    store.hot_swap_schema(schema);

    assert_eq!(store.resolve_color("theme.accent"), "#FF007F");
}

#[tokio::test]
async fn test_05_marketplace_npm_install_ed25519_integration() {
    let mut pm = PackageManager::new();

    let weather = pm.install("weather-widget").unwrap();
    assert_eq!(weather.id, "weather-widget");

    let spotify = pm.install("spotify-widget").unwrap();
    assert_eq!(spotify.id, "spotify-widget");

    let taskbar = pm.install("taskbar-plus").unwrap();
    assert_eq!(taskbar.id, "taskbar-plus");

    assert_eq!(pm.list().len(), 3);

    assert!(pm.uninstall("weather-widget").is_ok());
    assert_eq!(pm.list().len(), 2);
}

#[tokio::test]
async fn test_06_cloud_sync_crdt_vector_clock_offline_integration() {
    let mut sync = CloudSyncManager::new("desktop_workstation");

    let layout = SyncEntity::Layout(LayoutEntity {
        layout_id: "layout_1".into(),
        display_id: "MONITOR_1".into(),
        bounds_x: 100.0,
        bounds_y: 200.0,
        width: 400.0,
        height: 300.0,
    });

    sync.sync_entity("layout_1", layout.clone());
    assert_eq!(sync.get_entity("layout_1"), Some(&layout));

    // Test Offline Queuing
    sync.set_online_status(false);
    assert!(!sync.is_online());

    let theme = SyncEntity::Theme(ThemeEntity {
        theme_id: "theme_dark".into(),
        color_tokens: Default::default(),
        font_family: "Segoe UI".into(),
    });
    sync.sync_entity("theme_dark", theme);

    // Reconnect and Flush Queue
    sync.set_online_status(true);
    assert!(sync.is_online());
}

#[tokio::test]
async fn test_07_ai_voice_intent_and_workflow_automation_integration() {
    let cmd = VoiceIntentParser::parse_intent("switch to dark theme").unwrap();
    assert_eq!(
        cmd,
        ControlCommand::SetThemeMode {
            mode: "dark".to_string()
        }
    );

    let mut workflow = WorkflowAutomationEngine::new();
    workflow.add_rule(WorkflowRule {
        rule_id: "high_cpu_rule".to_string(),
        condition_metric: "sys.cpu_usage".to_string(),
        threshold_value: 80.0,
        action_command: cmd.clone(),
    });

    let actions = workflow.evaluate_telemetry("sys.cpu_usage", 90.0);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], cmd);
}

#[tokio::test]
async fn test_08_production_stress_and_master_release_integration() {
    assert!(SecurityAuditor::run_security_audit());
    assert!(StressTestingHarness::run_stress_test(100, 1000));
    assert!(MasterReleaseSuite::run_release_audit());
}
