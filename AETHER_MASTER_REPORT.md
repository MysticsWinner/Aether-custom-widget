# Aether — Master Project Report

**Authoritative Technical Reference — Generated from Source Code Investigation**

> This report was produced by systematically reading Aether's source code, configuration, tests, and existing documentation.  
> Claims are tagged as **[Verified]** (confirmed from implementation), **[Documented]** (stated in docs, consistent with code), **[Inferred]** (strongly indicated but not explicitly documented), or **[Stale]** (documentation contradicts current implementation).

**Date**: 2026-09-13  
**Workspace Root**: `d:\Code\Aether-custom-widget\`  
**Version**: 0.6.0 (workspace manifest), self-described as v0.7.0 in some docs  

---

## 1. Project Overview

### 1.1 Purpose
Aether is a next-generation desktop customization platform for **Windows 11** (`x86_64` and `ARM64`). It renders hardware-accelerated widget overlays directly on the Windows desktop surface behind desktop icons, functioning as a modern replacement for legacy tools like Rainmeter.

### 1.2 Goals
- Ultra-low resource usage (< 0.1% idle CPU, < 25 MB RAM for 100 widgets) **[Documented]**
- Hardware-accelerated rendering via DirectComposition + Direct2D at 144 Hz+ **[Documented]**
- Zero-trust plugin sandboxing via Windows AppContainer **[Documented]**
- Enterprise-grade security (Ed25519 package signatures, Group Policy, audit logging) **[Documented]**
- Extensible widget SDK with Rust, Lua 5.4, and planned WASM runtimes **[Verified]**

### 1.3 Current State
- **Phase**: Described as "Phase 16 — Production Release Candidate" in `AGENTS.md`, but `main.rs` banner says "Phase 15". **[Stale — conflicting]**
- **Compilation**: `cargo check --workspace` passes with 0 errors and 6 warnings (all unused fields in showcase widget crates). **[Verified]**
- **Rust Test Count**: **362 tests** passing across all workspace crates and integration suites (`cargo test --workspace`). **[Verified]**
- **C# GUI Tests**: **54 tests** passing across ViewModel, IPC, and Telemetry suites (`dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj`). **[Verified]**
- **Total Automated Test Count**: **416 / 416 tests passing** (100% pass rate). **[Verified]**


### 1.4 Key Terminology
| Term | Definition |
|:---|:---|
| **Engine Daemon** | The headless Rust background process (`core_engine.exe`) that orchestrates all subsystems |
| **Subsystem** | A modular component implementing the `Subsystem` trait, registered with `SubsystemManager` |
| **Widget** | A desktop overlay visual element implementing `WidgetLifecycle`, rendered via `DrawCommand` batches |
| **SharedTelemetryCache** | Lock-free (scalar) / RwLock (snapshot) shared memory store for hardware metrics |
| **WorkerW** | The Windows desktop shell window behind desktop icons, used as the rendering target |
| **ControlCommand** | An IPC message enum sent over Named Pipes between clients and the engine daemon |

---

## 2. Repository Structure

### 2.1 Top-Level Layout
```
Aether-custom-widget/
├── .agents/AGENTS.md           # Project governance rules
├── Cargo.toml                  # Workspace manifest — 33 member crates [Verified]
├── launch.ps1                  # PowerShell launcher for daemon + TUI
├── README.md                   # Documentation portal and quick start
├── crates/                     # 33 Rust workspace crates
├── src_gui/                    # WinUI 3 C# GUI Dashboard + Tests
├── docs/                       # 14-domain documentation system
├── tests/                      # Cross-crate integration/system test suite
├── native/                     # C++ Win32 WorkerW hook DLLs
├── bindings/                   # Multi-language SDK bindings
├── benchmarks/                 # Performance benchmark data
├── dist/                       # Distribution/release artifacts
└── logs/                       # Runtime log output
```

### 2.2 Workspace Crates (33 Members) [Verified from Cargo.toml]

#### Core Infrastructure
| Crate | Path | Responsibility |
|:---|:---|:---|
| `core_engine` | `crates/core_engine` | Tokio async daemon; `Engine`, `SubsystemManager`, `EventBus`, `TaskScheduler`, IPC server, rendering pipeline, desktop overlay window |
| `system_providers` | `crates/system_providers` | Hardware metric collectors (CPU, RAM, GPU, Network, Battery, Audio, Process, Display topology, GPU VRAM, CPU topology, WASAPI FFT, crypto prices, network diagnostics) + `SharedTelemetryCache` + `TelemetryService` + `TickRateAdvisor` |
| `ipc_protocol` | `crates/ipc_protocol` | `ControlCommand` enum (60+ variants), `MetricPayload` struct, shared-memory ring buffer types |
| `widget_sdk` | `crates/widget_sdk` | `WidgetLifecycle` trait, `WidgetState` enum (13 variants), `RenderCanvas`, `BatchRenderCanvas`, `DrawCommand`, reactive signals, settings, frame scheduling, LRU resource cache, SVG, FrameArena |

#### Widget Runtime & Layout
| Crate | Path | Responsibility |
|:---|:---|:---|
| `widget_parser` | `crates/widget_parser` | TOML `widget.toml` manifest schema parser and validator |
| `layout_engine` | `crates/layout_engine` | Flexbox layout computation via `taffy` |
| `animation_engine` | `crates/animation_engine` | Easing curves, spring physics, timeline scheduling |
| `plugin_runtime` | `crates/plugin_runtime` | AppContainer sandbox supervisor, API version compatibility, memory guard |
| `lua_runtime` | `crates/lua_runtime` | Sandboxed Lua 5.4 scripting bridge with hot code reload |
| `theme_engine` | `crates/theme_engine` | 12-category design token system, JSON theme parser, hot-reload file watcher, cascading inheritance, token resolver |

#### Security & Enterprise
| Crate | Path | Responsibility |
|:---|:---|:---|
| `capability_broker` | `crates/capability_broker` | Permission broker, runtime capability tokens, `WidgetFirewall` network proxy, BLAKE3 integrity monitor |
| `package_manager` | `crates/package_manager` | Widget package installer with Ed25519 signature verification, marketplace catalog |
| `enterprise` | `crates/enterprise` | Group Policy engine, SHA-256 tamper-evident audit logger, `AuthGate` Windows Hello biometric prompt |

#### Reliability & Diagnostics
| Crate | Path | Responsibility |
|:---|:---|:---|
| `recovery_manager` | `crates/recovery_manager` | Crash recovery, circuit breakers, Safe Mode sentinel, quarantine management |
| `config_manager` | `crates/config_manager` | Transactional atomic config persistence with 5-generation rolling backups, snapshot manager |
| `watchdog` | `crates/watchdog` | Heartbeat supervisor daemon |
| `event_recorder` | `crates/event_recorder` | Time-travel event stream recorder and replayer |
| `observability` | `crates/observability` | Prometheus metrics exporter, ETW tracing provider, minidump writer, flight recorder |
| `dev_tools` | `crates/dev_tools` | File-watcher hot-reloader, widget inspector, layout grid overlay, `aether` CLI |

#### Cloud, AI & Production
| Crate | Path | Responsibility |
|:---|:---|:---|
| `cloud_sync` | `crates/cloud_sync` | CRDT-based config sync with Lamport vector clocks, offline SQLite WAL cache, AES-256-GCM encryption |
| `ai_engine` | `crates/ai_engine` | Natural language widget/theme synthesizer, wallpaper theme generator, AI performance advisor |
| `production_engine` | `crates/production_engine` | Security auditor, stress harness, auto-updater, chaos testing, master release suite |

#### First-Party Showcase Widgets (8 crates)
| Crate | Responsibility |
|:---|:---|
| `perf_monitor_widget` | CPU/GPU/RAM glassmorphism card |
| `weather_widget` | Multi-city weather forecast |
| `network_monitor_widget` | Network adapter throughput |
| `ai_assistant_widget` | Conversational AI assistant |
| `audio_visualizer_widget` | WASAPI FFT audio spectrum + SMTC |
| `hardware_pro_widget` | GPU VRAM + CPU core topology matrix |
| `dock_launcher_widget` | Desktop application launcher dock |
| `weather_particles_widget` | Atmospheric particle effects (rain/snow) |
| `crypto_stocks_widget` | Financial market price ticker |

#### Build & Deploy
| Crate | Path | Responsibility |
|:---|:---|:---|
| `installer` | `crates/installer` | Windows setup wizard packaging compiled binaries into `%LOCALAPPDATA%\Aether\` |
| `dashboard_tui` | `crates/dashboard_tui` | Ratatui terminal dashboard — polls IPC pipe, renders animated gauges |

### 2.3 C# WinUI 3 Dashboard
- **Project**: `src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj` (.NET 8, Windows App SDK)
- **Tests**: `src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj`
- **Pages**: 13 XAML pages — Overview, Widgets, Design Tokens, AI Composer, Diagnostics, Performance, Marketplace, Profiles, Security, Services, Settings, Snapshots, About
- **Architecture**: MVVM with ViewModels, Services, Converters, IPCClient, and Styles directories

---

## 3. Architecture

### 3.1 System Layers [Verified from source]

```
┌──────────────────────────────────────────────────────────────────┐
│ 1. UI & Management Layer                                        │
│    ├── WinUI 3 Dashboard (C# / .NET 8) — 13 MVVM pages         │
│    └── Ratatui TUI Dashboard (Rust) — animated gauges           │
├──────────────────────────────────────────────────────────────────┤
│ 2. IPC & Serialization Layer                                    │
│    ├── Named Pipe: \\.\pipe\CustomWidgetEngineControlPipe       │
│    └── Wire Protocol: serde_json (ControlCommand / MetricPayload)│
├──────────────────────────────────────────────────────────────────┤
│ 3. Core Engine Daemon Layer (crates/core_engine)                │
│    ├── Engine — state machine (Initializing→Running→Paused→Stopped)│
│    ├── SubsystemManager — registration, cadence scheduling,     │
│    │   failure-aware rollback, execution stats, deadline tracking │
│    ├── EventBus — broadcast pub/sub with replay buffer          │
│    ├── TaskScheduler — adaptive power governor                  │
│    ├── IPC Server — tokio async Named Pipe dispatch             │
│    └── Subsystems: Telemetry, Render, Theme, Plugin, Profiler,  │
│        Marketplace, CloudSync, AI, Production                   │
├──────────────────────────────────────────────────────────────────┤
│ 4. Hardware & OS Layer                                          │
│    ├── SharedTelemetryCache (atomic scalars + RwLock snapshots)  │
│    ├── Win32 APIs: GetSystemTimes, GlobalMemoryStatusEx,        │
│    │   GetIfTable2, GetSystemPowerStatus, DXGI, EnumProcesses   │
│    ├── Desktop Shell: WorkerW / DesktopSurfaceManager           │
│    └── DirectComposition + Direct2D rendering pipeline          │
└──────────────────────────────────────────────────────────────────┘
```

### 3.2 Core Principle — "Collect Once, Publish Everywhere" [Verified]

The `TelemetrySubsystem` wraps `TelemetryService`, which samples all hardware metric providers once per engine tick. Results are committed to `SharedTelemetryCache`. All widgets and IPC clients read from the cache — zero repeated OS API calls per widget.

**Verified providers** (from `crates/system_providers/src/providers/mod.rs`):
- `CpuProvider` — Win32 `GetSystemTimes` with EMA smoothing (α=0.25), sub-quantum tick holding
- `MemoryProvider` — Win32 `GlobalMemoryStatusEx` with EMA smoothing (α=0.3)
- `GpuProvider` — DXGI `IDXGIAdapter3::QueryVideoMemoryInfo` with EMA smoothing
- `NetworkProvider` — Win32 `GetIfTable2` with time-scaled throughput and EMA smoothing
- `BatteryProvider` — Win32 `GetSystemPowerStatus`
- `AudioProvider` — **STUB: returns hardcoded 75% volume** (see BUG-001)
- `ProcessMetricsProvider` — Win32 `EnumProcesses` with heuristic category estimation (see BUG-002)
- `DisplayTopologyProvider` — Win32 `GetSystemMetrics(SM_CMONITORS)` + DXGI adapter enumeration
- `CpuTopologyProvider` — `GetLogicalProcessorInformationEx` for P/E core topology
- `DedicatedGpuProvider` — D3DKMT + DXGI for dedicated VRAM and 3D engine utilization
- `WasapiAudioProvider` — WASAPI loopback for 16-band FFT + SMTC media session
- `CryptoFinancialProvider` — Simulated crypto price data
- `NetworkDiagnosticsProvider` — Extended ping latency and bandwidth diagnostics

### 3.3 Subsystem Trait & Scheduling [Verified from subsystems.rs]

Every engine module implements:
```rust
#[async_trait]
pub trait Subsystem: Send + Sync {
    fn name(&self) -> &'static str;
    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()>;
    async fn tick(&mut self) -> anyhow::Result<()>;
    async fn shutdown(&mut self) -> anyhow::Result<()>;
    fn health(&self) -> SubsystemHealth;           // Healthy | Degraded | Failed
    fn scheduling_cadence(&self) -> SchedulingCadence; // Periodic | Reactive | OnDemand | DeadlineDriven
}
```

**Default cadence**: `Periodic(50ms)`. The `RenderSubsystem` uses `DeadlineDriven`. Theme, Plugin, AI, Cloud, and similar subsystems use `Reactive` or `OnDemand` — they only tick on explicit IPC triggers.

**Initialization rollback**: If any subsystem fails `initialize()`, all previously initialized subsystems are shut down in reverse registration order. **[Verified — tested in `test_partial_initialization_rollback_on_failure`]**

**Deadline miss tracking**: Each subsystem tick is timed. If a `Periodic` subsystem exceeds its declared period, a deadline miss is recorded in `SubsystemExecutionStats`. **[Verified — tested in `test_subsystem_deadline_miss_tracking`]**

### 3.4 Event Bus [Verified from event_bus.rs]

The `EventBus` uses `tokio::sync::broadcast` for pub/sub with:
- **Event classification**: `Ephemeral` (TelemetryTick — lossy), `Replayable` (state changes — buffered in circular replay buffer), `Durable` (ControlCommand — persisted)
- **Replay buffer**: Bounded `VecDeque<SequencedEvent>` with monotonic sequence numbers for client reconnection recovery
- **Gap detection**: `ReplayResult::GapDetected` returns an `AuthoritativeStateSnapshot` when requested history was pruned

### 3.5 SharedTelemetryCache — Hybrid Concurrency Model [Verified]

**Scalar fast path (wait-free)**: Individual metrics are stored as `AtomicU32` (for `f32` via `to_bits`/`from_bits`) and `AtomicU64`, using `Release`/`Acquire` ordering. Readers call `get_cpu_pct()` etc. with zero lock contention.

**Snapshot path (RwLock)**: Full `TelemetrySnapshot` structs (with GPU telemetry, audio spectrum, crypto assets, etc.) are stored behind `Arc<RwLock<TelemetrySnapshot>>`. This is NOT lock-free. **[Note: Documentation overstates this as "double-buffered seqlock" — see AC-001]**

**Sequence counter**: Monotonic `AtomicU64` incremented on each update for stale snapshot detection.

---

## 4. Runtime Behavior

### 4.1 Startup Sequence [Verified from main.rs]

1. Initialize `tracing_subscriber` with env filter (core_engine=info, perf_monitor_widget=info, dashboard_tui=info)
2. Emit ETW event 1001 ("Core Daemon Launch Initiated")
3. Run chaos engineering failure injection test (`FailureInjector`)
4. **Run 9 benchmark suites synchronously** (Render, Telemetry, SDK, Theme, Sandbox, PackageManager, CloudSync, AI, MasterRelease) — **this adds latency to every startup [AC-003]**
5. Create `EngineConfig` (10ms tick, 1024 event channel capacity, telemetry enabled)
6. Create `Engine` and register 9 subsystems: Telemetry, Render, Theme, PluginSandbox, Profiler, Marketplace, CloudSync, AI, Production
7. Call `engine.start()` → `SubsystemManager::initialize_all()` (sequential, with rollback on failure)
8. Spawn event monitor task (logs theme changes)
9. Create `DesktopWidgetWindow`, spawn desktop overlay thread
10. Create `IpcSharedState` with all service handles, spawn async IPC Named Pipe server
11. Enter 10ms tick loop (`tokio::select!` with Ctrl+C handler)

### 4.2 Tick Loop [Verified]

Every 10ms, `engine.tick()` calls `SubsystemManager::tick_scheduled()`, which:
1. Iterates all registered subsystems
2. Skips subsystems with `SubsystemHealth::Failed`
3. For `Periodic` cadence: only ticks if elapsed time ≥ declared period
4. For `DeadlineDriven`: always ticks
5. For `Reactive` / `OnDemand`: never ticks (must be triggered explicitly via `tick_subsystem()`)
6. Records execution statistics (tick count, duration, deadline misses)
7. On tick error: marks subsystem as `Degraded`

### 4.3 Shutdown [Verified from engine.rs]

1. `TaskScheduler::cancel_all()` — cancels background tasks
2. `SubsystemManager::shutdown_all()` — iterates subsystems in **reverse registration order**, calling `shutdown()` on each
3. Sets `EngineState::Stopped`
4. Publishes `CoreEvent::SystemStateChanged { state: "Stopped" }`
5. Emits ETW event 1003 ("Core Daemon Shutdown Complete")

### 4.4 IPC Communication [Verified from ipc_server.rs]

- **Transport**: Win32 Named Pipe at `\\.\pipe\CustomWidgetEngineControlPipe`
- **Protocol**: Newline-delimited JSON over the pipe. Each message is a serialized `ControlCommand`.
- **Dispatch**: Each pipe connection is handled in a separate `tokio::spawn` task. The handler deserializes the command, matches against the 60+ `ControlCommand` variants, executes the operation using `IpcSharedState` service handles, and writes a JSON response back.
- **Key services available via IPC**: telemetry queries, widget load/unload/toggle, crash history, safe mode, quarantine, snapshots, capability tokens, event recording, observability, chaos injection, marketplace search, enterprise policy, design tokens, AI synthesis, widget inspection, hot reload, and more.

---

## 5. Rendering Pipeline [Verified from rendering/ module]

### 5.1 Primary Path: DirectComposition + Direct2D
- `Direct2DRenderer` implements the `GpuRenderer` trait
- `DCompVisualTreeManager` manages DirectComposition visual nodes
- Renders to `WorkerW` desktop shell window via `DesktopSurfaceManager`
- `GlassmorphismPipeline` provides Mica/Acrylic/Frosted Glass shader passes
- `DirtyRegionTracker` tracks invalidated regions to minimize redraw area

### 5.2 Fallback Path: Layered HWND
- `DesktopWidgetWindow` supports `UpdateLayeredWindow` GDI fallback when DirectComposition is unavailable

### 5.3 Desktop Widget Window
- `DesktopWidgetWindow` (32,288 bytes — the largest source file) manages:
  - Widget overlay spawn thread with rendering loop
  - Virtual desktop awareness (`VirtualDesktopManager`)
  - Multi-monitor DPI scaling (`DpiMonitorScale`)
  - Particle effects engine (`ParticlePhysicsEngine`)
  - Per-widget position, opacity, scale, lock, and enable settings via `WidgetConfigStore`

---

## 6. Security Model [Verified from source + docs]

### 6.1 Plugin Sandboxing
- Third-party plugins execute in out-of-process Windows AppContainer sandboxes **[Documented]**
- JobObject resource quotas: 2% CPU, 50 MB RAM per plugin **[Documented]**
- Process mitigation policies: Win32k syscall disable, remote image load block, extension point disable **[Documented]**
- `CapabilityBroker` issues and verifies revocable runtime capability tokens
- `WidgetFirewall` acts as a network proxy for sandbox-to-network communication

### 6.2 Package Security
- Ed25519 cryptographic signature verification via `ed25519-dalek` for widget packages
- `AuditLogger` provides SHA-256 block hash-chained tamper-evident audit logs
- `AuthGate` supports Windows Hello biometric authentication prompts

### 6.3 Enterprise Governance
- `PolicyEngine` enforces Group Policy rules (MDM-compatible)
- Remotely configurable widget allow/deny lists

---

## 7. State & Persistence

### 7.1 Configuration
- `config_manager`: Transactional atomic config persistence with 5-generation rolling backups
- `SnapshotManager`: Desktop layout and state snapshots (create, restore, export, import)
- `WidgetConfigStore`: Per-widget position, opacity, scale, lock state (persisted JSON)

### 7.2 Persistent State
- Widget configuration: JSON files in `%LOCALAPPDATA%\Aether\`
- Theme definitions: JSON theme files
- Audit logs: SHA-256 hash-chained append-only log
- Crash history: Managed by `RecoveryManager`
- Cloud sync: Local SQLite WAL cache for CRDT state

### 7.3 Recovery
- `RecoveryManager`: Tracks consecutive crashes; triggers Safe Mode after threshold
- Quarantine: Widgets exceeding crash limits are permanently isolated until user intervention
- `DesktopSurfaceManager`: Monitors Explorer.exe lifecycle for shell restart recovery

---

## 8. Build & Deployment

### 8.1 Build Commands [Verified]
```powershell
# Workspace compilation check:
cargo check --workspace

# Build release binaries (with LTO):
cargo build --workspace --release

# Run all Rust tests:
cargo test --workspace

# Build C# dashboard:
dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj

# Run C# tests:
dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj
```

### 8.2 Runtime Binaries
- `core_engine.exe` — background daemon
- `dashboard_tui.exe` — terminal dashboard
- `installer.exe` — setup wizard
- `CustomWidget.Dashboard.exe` — WinUI 3 GUI (requires Windows App SDK runtime)

### 8.3 Prerequisites
- Rust 1.78+ (stable-x86_64-pc-windows-msvc)
- .NET 8.0 SDK + Visual Studio 2022 (Windows App SDK 1.5 workload) for the GUI dashboard
- Windows 11 Build 22000+

### 8.4 Launch
```powershell
.\launch.ps1                    # Core daemon (+ optional -IncludeTui / -DashboardOnly)
cargo run -p core_engine        # Manual daemon start
cargo run -p dashboard_tui      # Manual TUI start
```

---

## 9. Testing [Verified]

### 9.1 Test Infrastructure
- **Rust tests**: Standard `#[test]` and `#[tokio::test]` in each crate's source files
- **Integration tests**: `tests/integration_tests.rs` — cross-crate IPC, subsystem interaction
- **System tests**: `tests/system_tests.rs` — lifecycle, chaos recovery, layout persistence
- **Interface tests**: `tests/interface_tests.rs` — serialization compatibility, error handling
- **Release audit tests**: `tests/master_release_audit_tests.rs` — comprehensive release verification
- **C# tests**: `src_gui/CustomWidget.Dashboard.Tests/` — ViewModel and IPC client tests

### 9.2 Current Test Count
- **Rust**: 362 tests (from `cargo test --workspace -- --list`) **[Verified]**
- **C#**: 30 tests **[Documented — not independently verified]**
- **Total**: ~392

### 9.3 Test Naming Convention
```rust
#[test]
fn test_<unit_under_test>_<scenario>() { ... }
```

---

## 10. Current Problems

### 10.1 Confirmed Bugs
See [docs/Bugs/BUGS.md](docs/Bugs/BUGS.md) for details:
- **BUG-001**: `AudioProvider` returns hardcoded values (no WASAPI volume query)
- **BUG-002**: `ProcessMetricsProvider` uses heuristic estimation
- **BUG-003**: `TelemetrySnapshot::default()` contains misleading non-zero values
- **BUG-004**: Stale test counts across documentation

### 10.2 Technical Debt
See [docs/Bugs/TECH_DEBT.md](docs/Bugs/TECH_DEBT.md) for details:
- **TD-001**: GDI fallback needs DX11 SwapChain migration
- **TD-003**: Hardcoded mock values in providers
- **TD-005**: WorkerW unhook on resolution change (High severity)
- **TD-006**: `WidgetState` has 13 variants but docs say 11
- **TD-007**: Unused field warnings in showcase widget crates

### 10.3 Architectural Concerns
See [docs/Bugs/ARCHITECTURE_CONCERNS.md](docs/Bugs/ARCHITECTURE_CONCERNS.md):
- **AC-001**: `SharedTelemetryCache` uses RwLock for snapshots but docs claim fully lock-free
- **AC-002**: `ControlCommand` enum has 60+ flat variants
- **AC-003**: Benchmarks run synchronously on every daemon startup

### 10.4 Documentation Accuracy Issues
Multiple documents contain contradictory or stale information:
- Crate counts vary: "17 members" (PROJECT_OVERVIEW.md) vs 33 actual
- Test counts vary: 121 (AGENTS.md) / 313 / 333 / 343 (various docs) vs 362 actual
- Phase number: "Phase 16" (AGENTS.md) vs "Phase 15" (main.rs banner)
- Widget state count: "11 states" (ARCHITECTURE.md) vs 13 variants in code
- Lock-free claims: "double-buffered seqlock" vs hybrid atomic+RwLock reality
- Version: `0.6.0` in `Cargo.toml` workspace manifest vs `0.7.0` in project status docs

---

## 11. Documentation Inventory

### 11.1 Documentation Structure (14 domains, 90+ files)
```
docs/
├── AI/              — 5 files: Agent workflow, guidelines, limitations, task checklist, prompt library
├── API/             — 7 files: Rust SDK, C#, TypeScript, Lua, Plugin, IPC, Widget API references
├── Architecture/    — 12 files: Main architecture, ADRs (17 decisions), rendering, IPC, threading, memory, events, data flow
├── core/            — 12 files: Per-subsystem reference docs (Engine, Telemetry, Theme, Layout, etc.)
├── Development/     — 9 files: Build guide, coding standards, contributing, error handling, logging, workspace
├── GUI/             — 4 files: Dashboard, TUI, Diagnostics, Settings page docs
├── Performance/     — 4 files: Benchmarks, memory analysis, profiling, overview
├── Platform/        — 3 files: Windows, Linux (planned), macOS (planned)
├── Project/         — 9 files: Overview, status, changelog, roadmap, phases, features, plan 0.7, release notes
├── Release/         — 5 files: PR description, release process, versioning, changelog guide, production checklist
├── Rendering/       — 5 files: Direct2D, DirectComposition, dirty regions, GPU pipeline, WorkerW
├── Security/        — 5 files: Architecture, threat model, sandbox, permissions, signatures
├── Testing/         — 6 files: Main testing doc, structure, unit tests, integration, stress, QA checklist
├── Bugs/            — 5 files: README, BUGS, TECH_DEBT, ARCHITECTURE_CONCERNS, FUTURE_IDEAS (NEW)
└── archive/         — 1 file: Master release audit report
```

### 11.2 Key Accuracy Issues Found
| Document | Issue |
|:---|:---|
| `AGENTS.md` | Crate map lists only 17 of 33 crates; test count says 121; phase says 16 |
| `PROJECT_OVERVIEW.md` | Says "17 workspace crates" — actually 33 |
| `README.md` badge | Says "333/333 Passing" — actually 362 Rust tests |
| `Project_Status.md` | Says "343/343 (313 Rust + 30 C#)" — Rust is 362 |
| `Detailed_Project_Report.md` | Individual crate test counts don't sum to 313 |
| `ARCHITECTURE.md` | Claims "11 widget states" — `WidgetState` enum has 13 variants |
| `ARCHITECTURE.md` | Claims "double-buffered seqlock snapshots" — implementation uses `std::sync::RwLock` |
| `Cargo.toml` version | Says `0.6.0` — docs say `0.7.0` |

---

## 12. Dependencies [Verified from Cargo.toml]

| Category | Crate | Version |
|:---|:---|:---|
| **Async Runtime** | `tokio` | 1.38 (features: full) |
| **Windows API** | `windows` | 0.58 (26 feature flags) |
| **Serialization** | `serde` + `serde_json` | 1.0 |
| **TOML Parsing** | `toml` | 0.8 |
| **Logging** | `tracing` + `tracing-subscriber` | 0.1 / 0.3 |
| **Error Handling** | `anyhow` + `thiserror` | 1.0 |
| **Async Traits** | `async-trait` | (workspace) |
| **Cryptography** | `ed25519-dalek` | 2.1 |
| **Random** | `rand` | 0.8 |
| **Layout** | `taffy` | 0.4 |
| **Lua Scripting** | `mlua` | 0.9 (Lua 5.4, vendored) |
| **Futures** | `futures` | 0.3 |
| **TUI** | `ratatui` | 0.28 (dashboard_tui only) |
| **Terminal** | `crossterm` | 0.28 (dashboard_tui only) |

Release profile: `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`

---

## 13. Future Development

### 13.1 High Priority (from existing roadmap + investigation)
1. Move startup benchmarks behind `--benchmark` CLI flag (AC-003)
2. Implement real WASAPI volume in `AudioProvider` (BUG-001)
3. Migrate GDI overlay to DirectComposition DX11 SwapChain (TD-001)
4. Handle WorkerW unhook on resolution change (TD-005)
5. Add AMD ADL / NVIDIA NVML native GPU metrics

### 13.2 Medium Priority
6. Implement WASM (`wasmtime`) widget sandboxing runtime
7. Per-virtual-desktop layout profile persistence
8. `aether widget create <name>` scaffolding CLI
9. Windows Service wrapper for `core_engine`
10. Modularise `ControlCommand` enum (AC-002)

### 13.3 Long-Term
11. Cross-platform `DesktopSurface` trait (Linux Wayland / macOS Quartz)
12. ONNX local neural inference for adaptive layout optimization
13. Decentralized IPFS widget package storage
14. Active Directory GPO remote policy synchronization

---

*This report reflects the state of the Aether project as of 2026-09-13. Test counts and implementation details should be re-verified against the codebase when making architectural decisions.*
