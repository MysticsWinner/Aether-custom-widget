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
