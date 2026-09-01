# Aether — Release Notes v0.7.0 (Production Release Candidate)

**Official Release v0.7.0 (Production Release Candidate — Comprehensive GUI Dashboard & Memory Management)**  
**Date**: August 2026  
**Repository**: [https://github.com/MysticsWinner/Aether-custom-widget](https://github.com/MysticsWinner/Aether-custom-widget)

---

## 🚀 What's New in Version 0.7.0

Aether v0.7.0 introduces a complete, production-ready 13-page WinUI 3 management dashboard app, full reactive MVVM architecture via `CommunityToolkit.Mvvm`, automatic dependency process tree termination on app close, working-set memory management via Win32 `SetProcessWorkingSetSize`, an automated C# MSTest unit test suite, and deep IPC protocol integration across all 17 Rust engine crates.

### 🌟 Key Highlights & Features

1. **Complete WinUI 3 13-Page Management Dashboard (`src_gui/CustomWidget.Dashboard`)**:
   - **Overview Page**: Real-time telemetry gauges (CPU, GPU, RAM, NET), system status cards, quick action controls.
   - **Design Tokens Inspector Page (`DesignTokensPage` / `DesignTokensViewModel`)**: Interactive resolution and visual inspection of Aether 7.4 12-category semantic design token hierarchy (`Colors`, `Typography`, `Materials`, `Motion`).
   - **Desktop Profiles Manager Page (`ProfilesPage` / `ProfilesViewModel`)**: Context-aware activity profile switcher (`Coding`, `Gaming`, `Minimalist`, `Battery Saver`, `Creative Studio`) with live target FPS and material specification indicators.
   - **AI Desktop Composer Page (`AiComposerPage` / `AiComposerViewModel`)**: Natural language workstation layout synthesis with theme extraction (`cyberpunk`, `minimalist`, `aero`), material spec previews, resource footprint estimates, and preset prompt chips.
   - **Marketplace Package Store (`MarketplacePage` / `MarketplaceViewModel`)**: Cryptographically verified widget package store with Ed25519 signature validation badges, category filtering, live search query filtering, and 1-click Install/Uninstall toggles.
   - **Snapshots & System Recovery Hub (`SnapshotsPage` / `SnapshotsViewModel`)**: Transactional system configuration snapshot creation, restoration, deletion, and timestamp formatting.
   - **Security & Sandbox Visualizer (`SecurityPage` / `SecurityViewModel`)**: Real-time AppContainer process boundary monitor, active capability tokens, Job Object resource limits, and security audit log streams.
   - **Services Page (`ServicesPage` / `ServicesViewModel`)**: Subsystem worker daemon supervisor interface.
   - **Performance Page (`PerformancePage` / `PerformanceViewModel`)**: Frame timing and memory budget profiling.
   - **Diagnostics Page (`DiagnosticsPage` / `DiagnosticsViewModel`)**: Interactive IPC command console and raw JSON log viewer.
   - **Widgets Page (`WidgetsPage` / `WidgetsViewModel`)**: Widget lifecycle manager with position locking and display target pinning.
   - **Settings & About Pages (`SettingsPage`, `AboutPage`)**: Global telemetry refresh rate adjustments and system information.

2. **Automatic Dependency Shutdown & Memory Management (`MemoryManagerService`)**:
   - **Automatic Dependency Process Termination**: On app close (`MainWindow.Closed` & `AppDomain.CurrentDomain.ProcessExit`), `ProcessManagerService.StopEngineAsync()` kills `core_engine` daemon processes, cargo hosts, and entire child process trees with `entireProcessTree: true`.
   - **Physical RAM Working-Set Trimming**: Win32 P/Invoke `SetProcessWorkingSetSize` and forced garbage collection (`GC.Collect()`) reclaim physical RAM back to Windows OS when the app is closed or during periodic 30-second idle cleanups.

3. **C# MSTest GUI Automated Test Suite (`src_gui/CustomWidget.Dashboard.Tests`)**:
   - Built a dedicated 23-test C# unit test suite targeting `.NET 8` / Windows App SDK, covering all ViewModels, formatted property getters, IPC command serialization, and memory management lifecycle operations.

4. **Multi-Crate Rust Engine Capabilities**:
   - `theme_engine`: Design token resolution, WCAG 2.1 AA contrast calculations, Material degradation pipelines.
   - `ai_engine`: AI layout synthesis, voice command intent parsing, workflow automation.
   - `ipc_protocol`: Typed serde JSON serialization for `SearchMarketplace`, `CreateSnapshot`, `ListSnapshots`, `RestoreSnapshot`, `DeleteSnapshot`, `GetSecurityAuditLogs`, `ResolveDesignTokens`, `SetDesktopProfile`.

---

## 🛠️ Verification & Test Metrics

- **C# GUI Unit Test Suite**: **23 / 23 tests passing** (`dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj`).
- **Rust Backend Workspace Tests**: **268 / 268 tests passing** (`cargo test --workspace`).
- **Total Combined Test Suite**: **291 / 291 tests passing (100% pass rate)**.
- **WinUI 3 GUI Build**: **0 Warnings, 0 Errors** (`dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj`).
- **Rust Compilation**: **0 Errors** (`cargo check --workspace`).

---

## 📜 Historical Release Notes

Aether v0.6.0 delivers real-time system diagnostics, process management, log streaming, expanded IPC control APIs, and a comprehensive end-to-end integration test suite across all 17 workspace crates and WinUI 3 C# management dashboard.

### 🌟 Key Highlights & Features

1. **WinUI 3 Real-Time Diagnostics & Process Manager**:
   - **Interactive Diagnostics Page**: High-performance telemetry visualization featuring CPU/RAM usage timelines, memory allocation gauges, process table, and live log stream viewer.
   - **Process Manager Service**: Enumerates running desktop processes with PID, working set memory, CPU %, and thread counts. Supports single-click process termination (`KillProcess`).
   - **Log Collector Service**: Real-time structured log streaming (`tracing` & WinUI 3 event log view) with severity filter toggles (`Info`, `Warn`, `Error`, `Debug`).

2. **Expanded IPC Control API Protocol (`ipc_protocol` & `core_engine`)**:
   - Added new `ControlCommand` variants:
     - `GetDiagnostics` — returns comprehensive engine diagnostic snapshot (active widgets, pipe latency, memory usage, telemetry rates).
     - `GetServiceStatus` — queries status of core engine worker subsystems.
     - `StartService` / `StopService` — dynamic subsystem lifecycle management over named pipes.
     - `GetSystemLogs` — retrieves buffered diagnostic logs over IPC pipe.
   - Fully asynchronous IPC server dispatch handlers with JSON serialization.

3. **WinUI 3 Dashboard GUI & Service Integration (`src_gui`)**:
   - **Overview Page Enhancements**: Added quick diagnostic metrics, system health status badges, and engine subsystem cards.
   - **Settings Page Upgrades**: Advanced telemetry refresh rate selectors, IPC pipe status monitor, log buffer size controls, and reset configuration options.
   - **Services ViewModel**: Complete Windows service and daemon worker supervisor interface.
   - **Widgets Page Enhancements**: Reload, pause, resume, and restart widget controls.

4. **Multi-Crate Integration & System Test Suite (`tests/`)**:
   - Integrated full test workspace containing `tests/integration_tests.rs`, `tests/interface_tests.rs`, and `tests/system_tests.rs`.
   - **Coverage**:
     - Subsystem lifecycle & Tokio runtime orchestration.
     - IPC ring buffer concurrency & protocol compatibility.
     - AppContainer sandbox fault isolation & crash recovery.
     - Theme hot-reloading & token resolution.
     - Package installation with Ed25519 signature verification.
     - Cloud sync CRDT vector clock LWW state resolution.
     - Chaos failure injection & cold restart state persistence.

---

## 🛠️ Verification & Test Metrics
- **Workspace Test Suite**: **121/121 tests passing** (100% pass rate across unit, integration, interface, system, and benchmark tests).
- **Real-World Black-Box 20x Lifecycle Stress Test**: Verified **20/20 PASS (100% pass rate)** for repeated `LoadWidget` $\rightarrow$ `GetStatus` presence check $\rightarrow$ `UnloadWidget` $\rightarrow$ `GetStatus` removal check cycles across all widgets (`weather_widget`, `network_monitor_widget`, `ai_assistant_widget`).
- **Widget Visibility & Rendering Fixes**: Resolved `DEF-AETH-001` and `DEF-AETH-002` (making desktop overlay window visible for ALL loaded widgets and preventing `perf_monitor` unload from hiding other widgets). Added granular stage-by-stage `[WIDGET_LIFECYCLE]` logging.
- **Workspace Build**: Zero compilation errors across all 28 Rust crates and C# WinUI 3 dashboard (`src_gui/CustomWidget.Dashboard`) on Windows 11 (`x86_64` & `ARM64`).



---

## 📜 Historical Release Notes

### Official Release v0.5.0 (Production Release Candidate)
- **Task Manager-Style Telemetry & EMA Smoothing**: Windows APIs (`GetSystemTimes`, `GlobalMemoryStatusEx`), EMA smoothing ($\alpha = 0.25$), zero-delta tick hold.
- **Interactive Drag-to-Position & Lock Store**: Win32 `WM_NCHITTEST` hit testing on desktop overlay cards, `WM_EXITSIZEMOVE` persistence to `.aether/widget_positions.json`.
- **WinUI 3 Dashboard & Theme Engine**: Visual theme switching, About Page, dynamic Windows system accent sync (`DwmGetColorizationColor`).
- **Lua Scripting & AI Synthesis**: `mlua` 5.4 host bindings, telemetry queries, AI layout synthesis.
- **CRDT Cloud Sync & Package Manager**: Vector clock CRDT position sync, Ed25519 signature verifier.
