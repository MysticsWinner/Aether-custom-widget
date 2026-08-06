# Aether — Release Notes v0.6.0

**Official Release v0.6.0 (Production Release Candidate — Diagnostics & Integration)**  
**Date**: August 2026  
**Repository**: [https://github.com/MysticsWinner/Aether-custom-widget](https://github.com/MysticsWinner/Aether-custom-widget)

---

## 🚀 What's New in Version 0.6.0

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
- **Workspace Test Suite**: **121/121 tests passing** (100% pass rate across unit, integration, interface, and system tests).
- **Workspace Build**: Zero compilation warnings / zero errors on Windows 11 (`x86_64` & `ARM64`).

---

## 📜 Historical Release Notes

### Official Release v0.5.0 (Production Release Candidate)
- **Task Manager-Style Telemetry & EMA Smoothing**: Windows APIs (`GetSystemTimes`, `GlobalMemoryStatusEx`), EMA smoothing ($\alpha = 0.25$), zero-delta tick hold.
- **Interactive Drag-to-Position & Lock Store**: Win32 `WM_NCHITTEST` hit testing on desktop overlay cards, `WM_EXITSIZEMOVE` persistence to `.aether/widget_positions.json`.
- **WinUI 3 Dashboard & Theme Engine**: Visual theme switching, About Page, dynamic Windows system accent sync (`DwmGetColorizationColor`).
- **Lua Scripting & AI Synthesis**: `mlua` 5.4 host bindings, telemetry queries, AI layout synthesis.
- **CRDT Cloud Sync & Package Manager**: Vector clock CRDT position sync, Ed25519 signature verifier.
