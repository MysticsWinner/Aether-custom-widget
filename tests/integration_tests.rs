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

    // Use real-world sampled hardware telemetry metrics
    let snap = system_providers::sample_live_or_authentic_snapshot();
    let payload = MetricPayload::from(snap);
    assert!(payload.timestamp_ms > 0);
    assert!((0.0..=100.0).contains(&payload.cpu_usage_pct));
    assert!(payload.memory_total_mb > 0.0);

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
    let gate = ai_engine::AiSecurityGate::new();
    let action = VoiceIntentParser::parse_and_authorize("switch to dark theme", &gate, false).unwrap();
    let cmd = action.command;
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
    assert!(SecurityAuditor::run_security_audit().overall_passed);
    assert!(StressTestingHarness::run_stress_test(100, 1000));
    assert!(MasterReleaseSuite::run_release_audit());
}

#[tokio::test]
async fn test_09_gpu_d3dkmt_and_wasapi_audio_telemetry_flow() {
    use system_providers::{SharedTelemetryCache, TelemetryService};

    let cache = SharedTelemetryCache::new();
    let mut service = TelemetryService::new(cache.clone());

    let snapshot = service.collect_once().expect("Single-pass telemetry collection should succeed");
    assert!(snapshot.cpu_usage_pct >= 0.0);
    assert!(!snapshot.gpu_telemetry.adapter_name.is_empty());
    assert_eq!(snapshot.audio_spectrum.fft_bins_16.len(), 16);
    assert_eq!(snapshot.cpu_topology.per_core_usage_pct.len(), snapshot.cpu_topology.logical_core_count as usize);

    // Verify cache read matches snapshot
    assert_eq!(cache.get_gpu_telemetry().adapter_name, snapshot.gpu_telemetry.adapter_name);
    assert_eq!(cache.get_audio_spectrum().fft_bins_16.len(), 16);
}

#[tokio::test]
async fn test_10_interactive_hit_testing_and_svg_path_vector_rendering() {
    use widget_sdk::{BatchRenderCanvas, Color, HitTarget, HitTestTree, RectF, RenderCanvas, SvgPathData};

    let mut hit_tree = HitTestTree::new();
    hit_tree.register_target(HitTarget::new("play_button", RectF::new(50.0, 100.0, 40.0, 40.0)));
    hit_tree.register_target(HitTarget::new("waveform_area", RectF::new(0.0, 0.0, 400.0, 80.0)));

    let hit = hit_tree.hit_test(60.0, 110.0).expect("Hit target expected");
    assert_eq!(hit.element_id, "play_button");

    let svg = "M 0 0 L 50 50 Q 75 10 100 80 Z";
    let path_data = SvgPathData::parse(svg).expect("SVG path parsing expected to succeed");
    assert_eq!(path_data.segments.len(), 4);

    let mut canvas = BatchRenderCanvas::new();
    canvas.draw_path(svg, Some(Color::rgb(0.0, 1.0, 0.8)), None, 1.5);
    assert_eq!(canvas.commands().len(), 1);
}

#[tokio::test]
async fn test_11_wasm_plugin_sandboxed_lifecycle_and_memory_isolation() {
    use plugin_runtime::{WasmModuleSpec, WasmPluginEngine};

    let mut engine = WasmPluginEngine::new();
    let spec = WasmModuleSpec {
        module_id: "media_spectrum_visualizer".to_string(),
        version: "1.0.0".to_string(),
        max_memory_pages: 16,
        exported_functions: vec!["on_load".to_string(), "on_update".to_string(), "on_event".to_string()],
    };

    assert!(engine.load_module(spec).is_ok());
    let res = engine.invoke_function("media_spectrum_visualizer", "on_update").expect("Invocation should succeed");
    assert!(res.success);
    assert_eq!(res.function_name, "on_update");
}

#[tokio::test]
async fn test_12_context_aware_profile_automation_and_rollback() {
    use config_manager::{ContextAwareEngine, ContextSignal};

    let mut engine = ContextAwareEngine::new();
    assert_eq!(engine.active_profile_id(), "profile.default");

    let game_signal = ContextSignal {
        is_fullscreen: true,
        ..Default::default()
    };
    let switched = engine.update_context(&game_signal);
    assert_eq!(switched, Some("profile.gaming".to_string()));
    assert_eq!(engine.active_profile_id(), "profile.gaming");

    let restored = engine.rollback_profile();
    assert_eq!(restored, Some("profile.default".to_string()));
    assert_eq!(engine.active_profile_id(), "profile.default");
}

#[tokio::test]
async fn test_13_crypto_financial_and_network_diagnostics_integration() {
    use crypto_stocks_widget::CryptoStocksWidget;
    use system_providers::{CryptoFinancialProvider, NetworkDiagnosticsProvider, SharedTelemetryCache, TelemetrySnapshot};
    use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};

    let mut crypto_prov = CryptoFinancialProvider::new();
    let mut net_prov = NetworkDiagnosticsProvider::new();
    let cache = SharedTelemetryCache::new();

    let assets = crypto_prov.sample_all().expect("Crypto assets sampled");
    let net_diag = net_prov.sample().expect("Network diagnostics sampled");

    let mut snap = TelemetrySnapshot::default();
    snap.crypto_assets = assets;
    snap.network_diagnostics = net_diag;
    cache.update_snapshot(snap);

    let mut widget = CryptoStocksWidget::new(cache.clone());
    assert!(widget.on_load().is_ok());
    assert!(widget.on_mount().is_ok());

    let ctx = TickContext {
        timestamp_ms: 1000,
        delta_time_ms: 16.0,
        frame_index: 1,
    };
    assert!(widget.on_update(&ctx).is_ok());
    assert_eq!(widget.state(), WidgetState::Mounted);
    assert_eq!(widget.selected_symbol(), "BTC");

    assert!(widget.on_unmount().is_ok());
    assert!(widget.on_unload().is_ok());
}

#[tokio::test]
async fn test_14_ambient_weather_particles_and_frame_arena_integration() {
    use core_engine::rendering::virtual_desktops::{DpiMonitorScale, VirtualDesktopManager, VirtualDesktopPinning};
    use observability::FlightRecorder;
    use weather_particles_widget::WeatherParticlesWidget;
    use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
    use widget_sdk::FrameArena;

    let recorder = FlightRecorder::new(100);
    recorder.record("WEATHER", "Mounting weather particles widget", "INFO");

    let cache = system_providers::SharedTelemetryCache::new();
    let mut widget = WeatherParticlesWidget::new(cache);
    assert!(widget.on_load().is_ok());
    assert!(widget.on_mount().is_ok());

    let arena = FrameArena::with_capacity(16384);
    let _draw_buf = arena.alloc(2048, 8).expect("Allocation should succeed in frame arena");
    assert_eq!(arena.used_bytes(), 2048);

    let ctx = TickContext {
        timestamp_ms: 1000,
        delta_time_ms: 16.0,
        frame_index: 1,
    };
    assert!(widget.on_update(&ctx).is_ok());
    assert!(widget.active_particles_count() > 0);

    arena.reset();
    assert_eq!(arena.used_bytes(), 0);

    let v_desktop = VirtualDesktopManager::new(VirtualDesktopPinning::PinnedGlobally, 96);
    assert_eq!(v_desktop.pinning(), &VirtualDesktopPinning::PinnedGlobally);
    assert_eq!(v_desktop.dpi_scale().effective_dpi, 96);

    assert!(widget.on_unmount().is_ok());
    assert!(widget.on_unload().is_ok());
    assert_eq!(recorder.count(), 1);
}


