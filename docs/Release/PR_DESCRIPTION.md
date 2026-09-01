# Aether v0.7.0 Release & Pull Request Description

## 📌 Executive Summary
This release delivers a comprehensive modernization and performance hardening of the **Aether WinUI 3 Desktop Management Dashboard** (`CustomWidget.Dashboard`), its testing suite (`CustomWidget.Dashboard.Tests`), and integration with the 33-crate Rust Core Engine. It introduces robust chunked Named Pipe IPC streaming, typed Serde command serialization, clean MVVM service abstractions, adaptive telemetry frequency scaling, working-set memory management, and expands automated test coverage to **358 total passing tests** (313 Rust backend tests + 45 C# GUI tests).

---

## 🚀 Key Improvements & Innovations

### 1. 📡 Named Pipe IPC Communication & Resilient Streaming
- **Dynamic Chunked Stream Reading** (`NamedPipeClient.cs`):
  - Replaced the fixed 16 KB single-pass read with a dynamic stream accumulator, enabling zero-truncation reception for large responses (security audits, complete widget manifest catalogs, diagnostics dumps).
- **Graceful Cancellation Support**:
  - Implemented `CancellationToken` propagation across all 35+ IPC asynchronous methods in `IAetherIpcService`, eliminating hanging background requests when switching views or shutting down.
- **Exact Serde Tagging Fidelity** (`ControlCommand.cs` & `AetherIpcService.cs`):
  - Unified all command serializers to strictly match Rust Serde external tagging conventions:
    - Unit variants: `"Ping"`, `"GetStatus"`, `"ListSnapshots"`, `"ListWidgets"`, `"GetDiagnostics"`, `"GetSubsystemHealth"`, `"ReloadAll"`, `"ToggleDesktopWidget"`.
    - Struct variants: `{"LoadWidget": {"manifest_path": "..."}}`, `{"SetWidgetPosition": {"widget_id": "...", "x": 10, "y": 20}}`, `{"CreateSnapshot": {"name": "..."}}`, `{"RestoreSnapshot": {"snapshot_id": "..."}}`.
- **Rich Hardware & Market DTO Expansion** (`EngineStatus.cs`):
  - Added strongly-typed DTOs for `CryptoAssetDto` (BTC, ETH, SOL, S&P 500), `NetworkDiagnosticsDto` (DNS latency, jitter, packet loss), `GpuTelemetryDto`, `CpuTopologyDto`, and `AudioSpectrumDto`.

---

### 2. 🏛️ MVVM Architecture & Dependency Inversion
- **Service Interfaces** (`src_gui/CustomWidget.Dashboard/Services/Interfaces/`):
  - Abstracted all concrete services behind testable, decoupled interfaces:
    - `IAetherIpcService`
    - `ITelemetryPollerService`
    - `IProcessManagerService`
    - `IMemoryManagerService`
    - `IWidgetSettingsService`
    - `ILogCollectorService`
- **Standardized Base ViewModel** (`ViewModelBase.cs`):
  - Provides unified lifecycle handling, `IsBusy` flags, `ErrorMessage`/`HasError` reactive properties, and automated `CancellationTokenSource` cleanup upon disposal.
- **Decoupled Event Bus** (`AetherMessenger.cs`):
  - Implemented a thread-safe, in-memory pub/sub message bus allowing ViewModels to react to system-wide events (`WidgetRegistryChangedMessage`, `SnapshotRestoredMessage`, `ThemeModeChangedMessage`, `EngineConnectionStateMessage`, `DesktopOverlayToggledMessage`) without direct coupling.

---

### 3. ⚡ Performance Optimization & Memory Management
- **Adaptive Telemetry Frequency Scaling** (`TelemetryPollerService.cs`):
  - Introduced adaptive polling rates: 500 ms when the dashboard is active and focused, automatically throttled to 2000 ms when inactive/minimized.
- **Window Activation Working-Set Trimming** (`MainWindow.xaml.cs` & `MemoryManagerService.cs`):
  - Connected `EmptyWorkingSet` memory management to WinUI 3 window deactivation events, keeping Dashboard RAM footprint under 45 MB when idling in the background.

---

### 4. 🧪 Automated Test Suite & Verification Matrix

```
================================================================================
AUTOMATED TEST COMPARISON & COVERAGE:
--------------------------------------------------------------------------------
Previous Release (v0.6.0):  343 Tests (313 Rust + 30 C# GUI)
Current Release (v0.7.0):   358 Tests (313 Rust + 45 C# GUI) -> +15 New Tests
--------------------------------------------------------------------------------
Pass Rate:                  100% (358 / 358 Passing)
Compilation Errors:         0
Compiler Warnings:          0
================================================================================
```

### New Test Suites:
1. `CustomWidget.Dashboard.Tests/AetherIpcProtocolTests.cs` (11 tests):
   - Validates JSON serialization fidelity for Serde unit and struct command variants.
   - Tests error JSON escaping and formatting.
2. `CustomWidget.Dashboard.Tests/ViewModelTests.cs` (4 tests):
   - Validates `OverviewViewModel` telemetry binding and status transitions.
   - Validates `PerformanceViewModel` 60-second rolling window metric accumulation.
   - Validates `DiagnosticsViewModel` real-time log badge counts and filtering.
   - Validates `AetherMessenger` subscription, dispatch, and unsubscribe lifecycle.

---

## 🔒 Security & Quality Assurance
- **AppContainer Sandboxing Verified**: Low integrity level execution enforcement (`S-1-16-4096`).
- **Cryptographic Signatures**: Ed25519 verification active across all widget package installations and marketplace transactions.
- **Strict Threading Model**: DispatcherQueue thread-safety verified across background pollers and UI rendering loops.

---

## 📦 Binary Installer & Packaging
- Locally generated installer: `cargo run -p installer` -> packages compiled binaries (`core_engine.exe`, `dashboard_tui.exe`, `CustomWidget.Dashboard.exe`, `AetherSetup.exe`) and assets into `%LOCALAPPDATA%\Aether\`.
