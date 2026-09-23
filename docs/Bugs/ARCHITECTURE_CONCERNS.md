# Architectural Concerns

*Design-level issues that may affect long-term maintainability or correctness.*

---

## AC-001: `SharedTelemetryCache` uses `std::sync::RwLock` for snapshot despite claiming lock-free

- **Severity**: Medium
- **Component**: `system_providers` — `crates/system_providers/src/shared_cache.rs`
- **Evidence**: Documentation (ARCHITECTURE.md, ADR-013) describes "double-buffered seqlock snapshots" and claims `SharedTelemetryCache` is fully lock-free. In reality, individual scalar reads (`get_cpu_pct`, `get_gpu_pct`, etc.) are genuinely wait-free via `AtomicU32`/`AtomicU64` registers. However, `get_snapshot()` acquires a `std::sync::RwLock` read lock, and `update_snapshot()` acquires a write lock. Extended reads (e.g., `get_gpu_telemetry()`, `get_audio_spectrum()`) also acquire the `RwLock`.
- **Actual Behavior**: The scalar fast path is truly wait-free. The full snapshot path uses a standard `RwLock`. This is a perfectly reasonable hybrid design, but the documentation overstates the lock-free claim.
- **Impact**: No immediate correctness issue, but misleading architectural documentation could lead to incorrect assumptions by contributors about concurrency properties.
- **Status**: Open — documentation should clarify the hybrid model.

---

## AC-002: `ControlCommand` enum has grown to 60+ variants in a single flat enum

- **Severity**: Low
- **Component**: `ipc_protocol` — `crates/ipc_protocol/src/messages.rs`
- **Evidence**: The `ControlCommand` enum contains 60+ variants spanning telemetry, widget management, diagnostics, marketplace, AI, enterprise, snapshots, observability, and more. This creates a very large pattern match in the IPC server dispatch handler (`ipc_server.rs` is 1336 lines).
- **Impact**: Adding new IPC commands requires modifying a large file. No immediate correctness risk, but may become unwieldy as the platform grows.
- **Proposed Improvement**: Consider grouping into sub-enums by domain (e.g., `WidgetCommand`, `DiagnosticsCommand`, `MarketplaceCommand`) while keeping the top-level enum as a routing wrapper.
- **Status**: Open — low priority, deferred.

---

## AC-003: `main.rs` runs benchmarks synchronously on startup

- **Severity**: Low
- **Component**: `core_engine` — `crates/core_engine/src/main.rs`
- **Evidence**: The `main()` function runs `RainmeterBenchmark`, `TelemetryBenchmark`, `SdkBenchmark`, `ThemeBenchmark`, `PluginSandboxBenchmark`, `PackageManagerBenchmark`, `CloudSyncBenchmark`, `AiEngineBenchmark`, and `MasterReleaseSuite` sequentially before starting the engine. This adds startup latency and is atypical for a production daemon.
- **Impact**: Increases cold start time beyond the documented "<45ms" claim. Benchmarks are useful for CI but arguably should not run on every daemon start in production.
- **Status**: Open — consider moving benchmarks behind a `--benchmark` CLI flag.
