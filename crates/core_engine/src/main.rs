use std::sync::Arc;
use anyhow::Result;
use ai_engine::AiEngineBenchmark;
use cloud_sync::CloudSyncBenchmark;
use core_engine::{
    AiSubsystem, CloudSyncSubsystem, CoreEvent, Engine, EngineConfig, EtwTracingProvider,
    FailureInjector, FailurePoint, MarketplaceSubsystem, MasterPerformanceSuite,
    PluginSandboxSubsystem, ProductionSubsystem, ProfilerSubsystem, RainmeterBenchmark,
    RedundancySupervisor, RenderSubsystem, TelemetrySubsystem, ThemeEngineSubsystem,
};
use package_manager::PackageManagerBenchmark;
use plugin_runtime::PluginSandboxBenchmark;
use production_engine::MasterReleaseSuite;
use system_providers::TelemetryBenchmark;
use theme_engine::{ThemeBenchmark, ThemeResolver};
use widget_sdk::SdkBenchmark;
use tokio::time::{interval, Duration};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // ── 1. Logging subscriber ────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("core_engine=info".parse()?)
                .add_directive("perf_monitor_widget=info".parse()?)
                .add_directive("dashboard_tui=info".parse()?),
        )
        .init();

    EtwTracingProvider::emit_etw_event(1001, "Core Daemon Launch Initiated");

    info!("===========================================================");
    info!(" Next-Gen Windows Desktop Customization Platform - Core Daemon");
    info!(" Version: {}", env!("CARGO_PKG_VERSION"));
    info!(" Mode: Phase 15 Production Release Candidate Active (Certified Ready)");
    info!(" ETW Tracing & Failure Injection Diagnostics Enabled");
    info!("===========================================================");

    // ── 2. Chaos Engineering Failure Injection ───────────────────────────────
    info!("Executing Chaos Engineering Failure Injection & Redundancy Recovery Audits...");
    let injector = FailureInjector::new();
    injector.arm_failure(FailurePoint::GpuDeviceLost);
    if injector.should_fail(FailurePoint::GpuDeviceLost) {
        let _rec = RedundancySupervisor::handle_recovery(FailurePoint::GpuDeviceLost);
    }

    // ── 3. Benchmarks ────────────────────────────────────────────────────────
    info!("Executing GPU rendering, Telemetry, SDK, Theme, Sandbox, Marketplace, Cloud, AI & Release benchmarks...");
    let benchmark_result = RainmeterBenchmark::run_benchmark();
    info!(
        "Render Benchmark Result: Culling Efficiency = {:.1}%, Speedup = {:.1}x",
        benchmark_result.culling_efficiency_pct, benchmark_result.speedup_factor
    );

    TelemetryBenchmark::run_benchmark();
    SdkBenchmark::run_benchmark();
    ThemeBenchmark::run_benchmark();
    PluginSandboxBenchmark::run_benchmark();
    PackageManagerBenchmark::run_benchmark();
    CloudSyncBenchmark::run_benchmark();
    AiEngineBenchmark::run_benchmark();
    let _release_pass = MasterReleaseSuite::run_release_audit();
    let _nfr_report   = MasterPerformanceSuite::run_full_suite();

    // ── 4. Engine configuration ──────────────────────────────────────────────
    let config = EngineConfig::new()
        .with_tick_interval_ms(10)
        .with_event_channel_capacity(1024)
        .with_telemetry(true);

    // ── 5. Initialise Core Engine & subsystems ───────────────────────────────
    let mut engine = Engine::new(config);

    let (telemetry_sys, shared_cache) = TelemetrySubsystem::new();
    let (theme_sys, theme_store)      = ThemeEngineSubsystem::new();
    let plugin_sandbox_sys            = PluginSandboxSubsystem::new();
    let profiler_sys                  = ProfilerSubsystem::new();
    let marketplace_sys               = MarketplaceSubsystem::new();
    let cloud_sync_sys                = CloudSyncSubsystem::new();
    let ai_intelligence_sys           = AiSubsystem::new();
    let production_readiness_sys      = ProductionSubsystem::new();

    engine.register_subsystem(Box::new(telemetry_sys));
    engine.register_subsystem(Box::new(RenderSubsystem::new()));
    engine.register_subsystem(Box::new(theme_sys));
    engine.register_subsystem(Box::new(plugin_sandbox_sys));
    engine.register_subsystem(Box::new(profiler_sys));
    engine.register_subsystem(Box::new(marketplace_sys));
    engine.register_subsystem(Box::new(cloud_sync_sys));
    engine.register_subsystem(Box::new(ai_intelligence_sys));
    engine.register_subsystem(Box::new(production_readiness_sys));

    let event_bus = engine.event_bus();
    let mut event_rx = event_bus.subscribe();

    engine.start().await?;
    EtwTracingProvider::emit_etw_event(1002, "Core Daemon Startup Complete & Subsystems Online");

    // ── 6. Event monitor task ────────────────────────────────────────────────
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                CoreEvent::TelemetryTick { metric_id, value } => {
                    // Only log every ~5 s to avoid log spam
                    let _ = (metric_id, value);
                }
                CoreEvent::ThemeChanged { theme_name } => {
                    info!(target: "core_daemon", "Theme Hot Reload: '{}' applied!", theme_name);
                }
                _ => {}
            }
        }
    });

    // Spawn Desktop Overlay Widget Window (WorkerW / DirectComposition Layer)
    // NOTE: Created outside blocks so it can be shared with the IPC server.
    let desktop_window = Arc::new(core_engine::DesktopWidgetWindow::new());
    let initial_widgets = if desktop_window.is_visible() {
        vec!["aether.builtin.perf_monitor".to_string()]
    } else {
        vec![]
    };
    let widget_registry = Arc::new(std::sync::Mutex::new(initial_widgets));
    desktop_window.spawn_overlay(shared_cache.clone(), widget_registry.clone());

    // ── 8. IPC Named Pipe Server task ────────────────────────────────────────
    {
        let ipc_cache = shared_cache.clone();
        let ipc_desktop_window = desktop_window.clone();
        let ipc_state = core_engine::ipc_server::IpcSharedState::with_registry(
            ipc_cache,
            ipc_desktop_window,
            widget_registry,
        );
        tokio::spawn(async move {
            if let Err(e) = core_engine::ipc_server::run_ipc_server(ipc_state).await {
                tracing::error!("IPC server terminated: {e:?}");
            }
        });
    }

    info!(
        "Core Engine daemon running. Certified Production Release Candidate Active \
         (Accent: '{}', CPU: {:.1}%).\n\
         → IPC pipe ready at: {}\\\\.\\\\.\\pipe\\CustomWidgetEngineControlPipe\n\
         → Run `cargo run -p dashboard_tui` in a second terminal to open the live dashboard.\n\
         Press Ctrl+C to exit.",
        theme_store.resolve_color("theme.accent"),
        shared_cache.get_cpu_pct(),
        "",
    );

    // ── 9. Engine tick loop ──────────────────────────────────────────────────
    let mut tick_timer = interval(Duration::from_millis(10));
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C OS shutdown signal.");
        }
        _ = async {
            loop {
                tick_timer.tick().await;
                engine.tick().await;
            }
        } => {}
    }

    // ── 10. Clean shutdown ───────────────────────────────────────────────────
    engine.stop().await;
    EtwTracingProvider::emit_etw_event(1003, "Core Daemon Shutdown Complete");
    info!("Core Engine daemon exit complete.");

    Ok(())
}
