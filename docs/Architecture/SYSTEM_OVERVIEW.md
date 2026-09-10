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
- **Responsibility**: Hardware metric sampling and wait-free cache updates.
- **Key Structs**: `TelemetryService`, `SharedTelemetryCache`, `CpuProvider`, `MemoryProvider`, `GpuProvider`, `NetworkProvider`, `CpuTopologyProvider`, `WasapiAudioProvider`.
- **Implementation Status**:
  - ✅ Real Win32 API for CPU (`GetSystemTimes`), RAM (`GlobalMemoryStatusEx`), Network (`GetIfTable2`), GPU (`D3DKMTEnumAdapters2`), and CPU topology (`GetLogicalProcessorInformationEx`).
  - ✅ Wait-free atomic scalar registers for zero-contention fast-path reads (`get_cpu_pct`, `get_memory_used_mb`).
  - ✅ Seqlock double-buffered snapshots (`TelemetrySnapshot`) for atomic composite reads.
  - ✅ Sub-quantum holding and EMA smoothing ($\alpha = 0.25$).

### 2.2 `RenderSubsystem`
- **Crate**: `crates/core_engine/src/rendering/`
- **Responsibility**: Authoritative DirectComposition visual tree hosting, Direct2D drawing, dirty rect tracking, WorkerW shell recovery, and per-monitor V2 DPI coordinate virtualization.
- **Key Structs**: `Direct2DRenderer`, `DCompVisualTreeManager`, `DesktopSurfaceManager`, `DirtyRegionTracker`, `VirtualDesktopManager`, `DesktopTopology`.
- **Implementation Status**:
  - ✅ DirectComposition + Direct2D device context swapchain target as primary authoritative pipeline.
  - ✅ Layered HWND (`UpdateLayeredWindow`) isolated strictly as compatibility fallback.
  - ✅ `DesktopSurfaceManager` tracking `WorkerWSurfaceState` with automated Explorer crash detection and Progman 0x052C rebind.
  - ✅ `DirtyRegionTracker` tracking `InvalidatedRegion` with `InvalidationCause` classification and bounding box capacity collapse.
  - ✅ Canonical DIP (`DipRect`) vs Physical display pixels (`PhysicalRect`) across heterogeneous multi-monitor setups.

### 2.3 `ThemeEngineSubsystem`
- **Crate**: `crates/theme_engine` & `crates/core_engine/src/theme_subsystem.rs`
- **Responsibility**: 12-category design token schema resolution, cascading inheritance (`Base -> Derived -> Widget -> Instance`), cycle detection, and hot-reloading.
- **Key Structs**: `ThemeResolver`, `ThemeSchema`, `MaterialSpec`, `DesignTokens`.
- **Implementation Status**: ✅ Implemented with JSON hot-reload watcher and token variable substitution.

### 2.4 `PluginSandboxSubsystem`
- **Crate**: `crates/plugin_runtime` & `crates/core_engine/src/plugin_subsystem.rs`
- **Responsibility**: Sandboxed plugin process supervision, AppContainer isolation, JobObject CPU/RAM limits, and crash isolation.
- **Key Structs**: `PluginSupervisor`, `PermissionManifest`, `CompatibilityChecker`, `WasmPluginEngine`.
- **Implementation Status**: ✅ Implemented with Win32 JobObject memory caps (64 MB), crash trapping, restart thresholds, and quarantine state machine.

### 2.5 `ProfilerSubsystem`
- **Crate**: `crates/core_engine/src/profiler_subsystem.rs`
- **Responsibility**: Subsystem latency tracking, NFR compliance verification, and ETW tracing emission.
- **Key Structs**: `SystemProfiler`, `MasterPerformanceSuite`, `PerformanceReport`.
- **Implementation Status**: ✅ Implemented with microsecond-level tick latency sampling.

### 2.6 `MarketplaceSubsystem`
- **Crate**: `crates/package_manager` & `crates/core_engine/src/marketplace_subsystem.rs`
- **Responsibility**: NPM-style widget package installation, dependency resolution, and Ed25519 signature verification.
- **Key Structs**: `PackageManager`, `WidgetPackage`, `Ed25519Verifier`.
- **Implementation Status**: ✅ Implemented with cryptographic verification and tar/gzip unpacking.

### 2.7 `CloudSyncSubsystem`
- **Crate**: `crates/cloud_sync` & `crates/core_engine/src/cloud_subsystem.rs`
- **Responsibility**: Multi-workstation state CRDT conflict resolution (Lamport Vector Clocks, Last-Write-Wins), AES-256-GCM encryption, and offline queues.
- **Key Structs**: `CloudSyncManager`, `VectorClock`, `CrdtResolver`, `OfflineSyncQueue`.
- **Implementation Status**: ✅ Implemented with deterministic conflict-free merging.

### 2.8 `AiSubsystem`
- **Crate**: `crates/ai_engine` & `crates/core_engine/src/ai_subsystem.rs`
- **Responsibility**: Desktop composition synthesis, voice command translation, workflow automation, and mandatory security validation gate.
- **Key Structs**: `AiDesktopComposer`, `VoiceIntentParser`, `AiSecurityGate`, `UntrustedAiProposal`, `WorkflowAutomationEngine`.
- **Implementation Status**: ✅ Implemented with 4-stage security gate (`SchemaValidator`, `PolicyValidator`, `CapabilityValidator`, `HumanApprovalGate`) blocking unauthorized commands and path traversal.

### 2.9 `ProductionSubsystem`
- **Crate**: `crates/production_engine` & `crates/core_engine/src/production_subsystem.rs`
- **Responsibility**: Automated security auditing, stress testing, auto-updater hash checks, and release verification.
- **Key Structs**: `SecurityAuditor`, `StressTestingHarness`, `AutoUpdater`, `MasterReleaseSuite`.
- **Implementation Status**: ✅ Implemented with comprehensive test suites and automated reports.

### 2.10 `FaultDiagnostics`
- **Crate**: `crates/core_engine/src/fault_diagnostics.rs`
- **Responsibility**: Controlled chaos failure injection, ETW tracing emission, and redundancy supervisor failover.
- **Key Structs**: `FailureInjector`, `EtwTracingProvider`, `RedundancySupervisor`.
- **Implementation Status**: ✅ Implemented with armed trigger execution and failover health checks.
