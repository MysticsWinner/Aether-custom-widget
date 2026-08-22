# Aether — System Overview & Subsystem Inventory

**Exhaustive Analysis of Core Subsystems**

---

## 1. Engine Subsystem Inventory

The Aether core engine manages 10 dedicated subsystems via `SubsystemManager` inside `crates/core_engine/src/subsystems.rs`:

```
core_engine
├── TelemetrySubsystem
├── RenderSubsystem
├── ThemeEngineSubsystem
├── PluginSandboxSubsystem
├── ProfilerSubsystem
├── MarketplaceSubsystem
├── CloudSyncSubsystem
├── AiSubsystem
├── ProductionSubsystem
└── FaultDiagnostics
```

---

## 2. Subsystem Specifications

### 2.1 `TelemetrySubsystem`
- **Crate**: `crates/system_providers` & `crates/core_engine/src/telemetry_subsystem.rs`
- **Responsibility**: Hardware metric sampling (CPU, RAM, GPU, Net) and central cache updates.
- **Key Structs**: `TelemetryService`, `SharedTelemetryCache`, `CpuProvider`, `MemoryProvider`, `GpuProvider`, `NetworkProvider`.
- **Status**: ✅ Real Win32 API for CPU (`GetSystemTimes`) & RAM (`GlobalMemoryStatusEx`); 🔶 Simulated for GPU & Network.

### 2.2 `RenderSubsystem`
- **Crate**: `crates/core_engine/src/rendering/`
- **Responsibility**: Surface invalidated regions, track dirty rectangles, calculate frame budgets, and composite draw command batches.
- **Key Structs**: `Direct2DRenderer`, `GpuRenderer` trait, `DirtyRegionTracker`, `RainmeterBenchmark`.
- **Status**: 🔶 Functional Skeleton (calculates dirty rect bounds and frame budgets in memory; desktop window painting pending).

### 2.3 `ThemeEngineSubsystem`
- **Crate**: `crates/theme_engine` & `crates/core_engine/src/theme_subsystem.rs`
- **Responsibility**: Theme JSON schema resolution, color token lookups (`Primary`, `Secondary`, `GlassBackground`), and theme hot-reloading.
- **Key Structs**: `ThemeResolver`, `ThemeSchema`, `ThemeHotReloadWatcher`.
- **Status**: 🔶 Functional Skeleton.

### 2.4 `PluginSandboxSubsystem`
- **Crate**: `crates/plugin_runtime` & `crates/core_engine/src/plugin_subsystem.rs`
- **Responsibility**: Widget plugin process supervision, API version checking (`CompatibilityChecker`), and capability permission verification.
- **Key Structs**: `PluginSupervisor`, `PermissionManifest`, `CompatibilityChecker`.
- **Status**: 🔶 Functional Skeleton (PID tracking and isolation checks simulated in memory).

### 2.5 `ProfilerSubsystem`
- **Crate**: `crates/core_engine/src/profiler_subsystem.rs`
- **Responsibility**: Latency tracking across 10ms engine tick execution cycles and subsystem diagnostics logging.
- **Status**: 🔶 Functional Skeleton.

### 2.6 `MarketplaceSubsystem`
- **Crate**: `crates/package_manager` & `crates/core_engine/src/marketplace_subsystem.rs`
- **Responsibility**: NPM-style widget package installation, uninstallation, package listing, and Ed25519 signature checks.
- **Key Structs**: `PackageManager`, `WidgetPackage`, `Ed25519Verifier`.
- **Status**: 🔶 Functional Skeleton.

### 2.7 `CloudSyncSubsystem`
- **Crate**: `crates/cloud_sync` & `crates/core_engine/src/cloud_subsystem.rs`
- **Responsibility**: Multi-device state CRDT conflict resolution (Vector Clocks, Last-Write-Wins) and offline synchronization queues.
- **Key Structs**: `CloudSyncManager`, `VectorClock`, `CrdtResolver`, `OfflineSyncQueue`.
- **Status**: 🔶 Functional Skeleton.

### 2.8 `AiSubsystem`
- **Crate**: `crates/ai_engine` & `crates/core_engine/src/ai_subsystem.rs`
- **Responsibility**: Synthetic layout, color palette, TOML widget manifest, and voice command processing.
- **Key Structs**: `LayoutGenerator`, `ThemeGenerator`, `WidgetGenerator`, `VoiceCommandProcessor`.
- **Status**: 🔶 Functional Skeleton (keyword matching engine).

### 2.9 `ProductionSubsystem`
- **Crate**: `crates/production_engine` & `crates/core_engine/src/production_subsystem.rs`
- **Responsibility**: Automated security auditing, load stress testing, auto-update verification, crash reporting, and docs portal generation.
- **Key Structs**: `SecurityAuditor`, `StressTestHarness`, `AutoUpdater`, `CrashAnalytics`, `MasterReleaseSuite`.
- **Status**: 🔶 Functional Skeleton.

### 2.10 `FaultDiagnostics`
- **Crate**: `crates/core_engine/src/fault_diagnostics.rs`
- **Responsibility**: Controlled failure injection for testing resilience, ETW tracing provider simulation, and subsystem redundancy supervision.
- **Key Structs**: `FailureInjector`, `EtwTracingProvider`, `RedundancySupervisor`.
- **Status**: 🔶 Functional Skeleton.
