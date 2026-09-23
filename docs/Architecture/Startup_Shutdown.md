# Startup & Shutdown Lifecycle

**Purpose**: Documents initialization sequence, subsystem startup order, and graceful reverse shutdown handling.  
**Audience**: Engine Engineers, System Integrators.  
**Prerequisites**: [ARCHITECTURE.md](ARCHITECTURE.md).  
**Related Documents**: [Engine.md](../Core/Engine.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Core Engine Team  

---

## 1. Sequential Startup Phase (Verified from `core_engine/src/main.rs`)

1. Initialize `tracing_subscriber` with env filter directives.
2. Emit ETW event 1001 ("Core Daemon Launch Initiated").
3. Execute chaos engineering failure injection test (`FailureInjector`).
4. Run 9 benchmark suites synchronously (Render, Telemetry, SDK, Theme, Sandbox, PackageManager, CloudSync, AI, MasterRelease). **Note: This adds startup latency — see AC-003 in `docs/Bugs/ARCHITECTURE_CONCERNS.md`.**
5. Create `EngineConfig` (10ms tick interval, 1024 event channel capacity, telemetry enabled).
6. Create `Engine` and register 9 subsystems: Telemetry, Render, Theme, PluginSandbox, Profiler, Marketplace, CloudSync, AI, Production.
7. Call `engine.start()` → `SubsystemManager::initialize_all()` — sequential initialization with reverse-order rollback on failure.
8. Spawn event monitor task (logs theme changes from `EventBus`).
9. Create `DesktopWidgetWindow` and spawn desktop overlay thread with `SharedTelemetryCache`.
10. Create `IpcSharedState` with all service handles and spawn async Named Pipe server on `\\.\pipe\CustomWidgetEngineControlPipe`.
11. Enter 10ms tick loop under `tokio::select!` with Ctrl+C handler.

---

## 2. Reverse-Order Shutdown Phase (Verified from `core_engine/src/engine.rs`)

On receiving SIGINT/Ctrl+C:

1. `TaskScheduler::cancel_all()` — cancel background tasks.
2. `SubsystemManager::shutdown_all()` — iterate subsystems in **reverse registration order**, calling `shutdown()` on each. Failures are logged but do not prevent subsequent shutdowns.
3. Set `EngineState::Stopped`.
4. Publish `CoreEvent::SystemStateChanged { state: "Stopped" }` to event bus.
5. Emit ETW event 1003 ("Core Daemon Shutdown Complete").

---

## Future Work
- Add fast crash dump flush handler during unhandled panic signals.
- Move startup benchmarks behind a `--benchmark` CLI flag to improve production cold start time.

## Known Issues
- Startup benchmarks add latency to every daemon start (see `docs/Bugs/ARCHITECTURE_CONCERNS.md` AC-003).

## References
- [crates/core_engine/src/main.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/main.rs)
- [crates/core_engine/src/engine.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/engine.rs)
- [crates/core_engine/src/subsystems.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/subsystems.rs)

## Related Documents
- [Engine.md](../core/Engine.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)

