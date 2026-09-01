//! System Integration Tests for the Aether Platform
//!
//! Validates end-to-end user workflows, lifecycle stages, fault-tolerance via
//! chaos failure injection, and widget layout state persistence across sessions.

use core_engine::{
    AiSubsystem, CloudSyncSubsystem, Engine, EngineConfig, EngineState, FailureInjector,
    FailurePoint, MarketplaceSubsystem, PluginSandboxSubsystem, ProductionSubsystem,
    ProfilerSubsystem, RedundancySupervisor, RenderSubsystem, TelemetrySubsystem,
    ThemeEngineSubsystem,
};
use layout_engine::WidgetPositionStore;

#[tokio::test]
async fn test_system_full_lifecycle_e2e() {
    let config = EngineConfig::new()
        .with_tick_interval_ms(10)
        .with_event_channel_capacity(1024)
        .with_telemetry(true);

    let mut engine = Engine::new(config);

    // Register all 9 subsystems
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

    // Verify engine initial state
    assert_eq!(engine.state().await, EngineState::Initializing);

    // Start engine
    assert!(engine.start().await.is_ok());
    assert_eq!(engine.state().await, EngineState::Running);

    // Simulate clock ticks (telemetry collection, render passes)
    for _ in 0..5 {
        engine.tick().await;
    }

    // Verify hardware telemetry metrics collection
    assert!(shared_cache.get_cpu_pct() >= 0.0);
    assert!(shared_cache.get_memory_used_mb() >= 0.0);

    // Stop engine gracefully and assert resources are released
    engine.stop().await;
    assert_eq!(engine.state().await, EngineState::Stopped);
}

#[tokio::test]
async fn test_system_chaos_failure_injection_recovery() {
    let config = EngineConfig::new()
        .with_tick_interval_ms(10)
        .with_telemetry(true);

    let mut engine = Engine::new(config);

    // Register essential subsystems
    let (telemetry_sys, _shared_cache) = TelemetrySubsystem::new();
    engine.register_subsystem(Box::new(telemetry_sys));
    engine.register_subsystem(Box::new(RenderSubsystem::new()));

    assert!(engine.start().await.is_ok());

    // Inject chaos fault - Simulate a DirectComposition GPU device lost event
    let injector = FailureInjector::new();
    injector.arm_failure(FailurePoint::GpuDeviceLost);

    assert!(injector.should_fail(FailurePoint::GpuDeviceLost));

    // Trigger redundancy supervisor recovery
    let recovery_status = RedundancySupervisor::handle_recovery(FailurePoint::GpuDeviceLost);
    assert!(recovery_status);

    // Ensure the engine continues ticking without crashing
    for _ in 0..3 {
        engine.tick().await;
    }

    engine.stop().await;
}

#[tokio::test]
async fn test_system_cold_restart_persistence() {
    let temp_path = std::env::temp_dir().join("test_system_cold_restart_persistence.json");
    let store = WidgetPositionStore::new(Some(temp_path.clone()));
    let widget_id = "aether.custom.weather";

    // 1. Simulate active session: set position coordinates and lock state
    assert!(store.set_position(widget_id, 250, 400).is_ok());
    assert!(store.set_locked(widget_id, true).is_ok());

    // Verify properties saved in store
    assert_eq!(store.get_position(widget_id).unwrap(), (250, 400));
    assert!(store.is_locked(widget_id));

    // 2. Simulate shutdown (store persistent state saved to temp file)
    drop(store);

    // 3. Simulate cold restart: instantiate a fresh position store pointing to same file
    let fresh_store = WidgetPositionStore::new(Some(temp_path.clone()));

    // Verify coordinates and locks are reloaded and restored correctly from layout JSON file
    let pos = fresh_store.get_position(widget_id);
    assert_eq!(pos, Some((250, 400)));
    assert!(fresh_store.is_locked(widget_id));

    // Clean up test side-effects
    let _ = std::fs::remove_file(temp_path);
}

#[tokio::test]
async fn test_system_adaptive_power_and_glassmorphism_pipeline_e2e() {
    use core_engine::rendering::glassmorphism::GlassmorphismPipeline;
    use core_engine::task_scheduler::{AdaptivePowerGovernor, PowerProfile};
    use theme_engine::{MaterialSpec, MaterialType};
    use widget_sdk::{BatchRenderCanvas, RectF};

    // 1. Test Adaptive Power Governor switching across system states
    let mut governor = AdaptivePowerGovernor::new();
    assert_eq!(governor.current_profile(), PowerProfile::Standard);

    // Interactive audio session -> HighPerformance (144 Hz)
    let p1 = governor.evaluate_profile(true, true, false, false, 0);
    assert_eq!(p1, PowerProfile::HighPerformance);
    assert_eq!(p1.target_refresh_rate_hz(), 144);

    // Fullscreen 3D game -> Suppressed (0 Hz)
    let p2 = governor.evaluate_profile(false, false, false, true, 0);
    assert_eq!(p2, PowerProfile::Suppressed);
    assert_eq!(governor.cpu_savings_estimate_pct(), 100.0);

    // 2. Test Glassmorphism Pipeline multi-pass shader composition
    use widget_sdk::RenderCanvas;
    let pipeline = GlassmorphismPipeline::new().with_dithering(true);
    let mut canvas = BatchRenderCanvas::new();
    let spec = MaterialSpec {
        material_type: MaterialType::Acrylic,
        tint_color: "#161B22".to_string(),
        tint_opacity: 0.90,
        blur_radius: 30.0,
        border_highlight: true,
        ..Default::default()
    };

    pipeline.apply_material(&mut canvas, widget_sdk::rendering::RectF::new(0.0, 0.0, 400.0, 200.0), &spec);
    assert!(canvas.commands().len() >= 2);
}

#[tokio::test]
async fn test_system_showcase_widgets_coexistence_e2e() {
    use audio_visualizer_widget::AudioVisualizerWidget;
    use dock_launcher_widget::DockLauncherWidget;
    use hardware_pro_widget::HardwareProWidget;
    use perf_monitor_widget::PerfMonitorWidget;
    use system_providers::{SharedTelemetryCache, TelemetryService};
    use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};

    let cache = SharedTelemetryCache::new();
    let mut service = TelemetryService::new(cache.clone());

    // 1. Initialize all 4 built-in showcase widgets
    let mut perf = PerfMonitorWidget::new(cache.clone());
    let mut audio = AudioVisualizerWidget::new(cache.clone());
    let mut hw_pro = HardwareProWidget::new(cache.clone());
    let mut dock = DockLauncherWidget::new(cache.clone());

    assert!(perf.on_load().is_ok());
    assert!(audio.on_load().is_ok());
    assert!(hw_pro.on_load().is_ok());
    assert!(dock.on_load().is_ok());

    assert!(perf.on_mount().is_ok());
    assert!(audio.on_mount().is_ok());
    assert!(hw_pro.on_mount().is_ok());
    assert!(dock.on_mount().is_ok());

    assert_eq!(perf.state(), WidgetState::Mounted);
    assert_eq!(audio.state(), WidgetState::Mounted);
    assert_eq!(hw_pro.state(), WidgetState::Mounted);
    assert_eq!(dock.state(), WidgetState::Mounted);

    // 2. Perform simultaneous telemetry sampling and multi-widget tick pass
    let _snapshot = service.collect_once().expect("Telemetry collection should succeed");
    let ctx = TickContext {
        timestamp_ms: 1000,
        delta_time_ms: 16.0,
        frame_index: 1,
    };

    assert!(perf.on_update(&ctx).is_ok());
    assert!(audio.on_update(&ctx).is_ok());
    assert!(hw_pro.on_update(&ctx).is_ok());
    assert!(dock.on_update(&ctx).is_ok());

    // 3. Gracefully tear down all widgets
    assert!(perf.on_unmount().is_ok());
    assert!(audio.on_unmount().is_ok());
    assert!(hw_pro.on_unmount().is_ok());
    assert!(dock.on_unmount().is_ok());

    assert!(perf.on_unload().is_ok());
    assert!(audio.on_unload().is_ok());
    assert!(hw_pro.on_unload().is_ok());
    assert!(dock.on_unload().is_ok());
}

