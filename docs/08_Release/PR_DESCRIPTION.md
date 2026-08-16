# Pull Request: Comprehensive Desktop Rendering Engine, Per-Widget Configuration, Quick Settings Flyout, WinUI 3 Dashboard (13 Pages) & Automated Test Suite

## 📌 PR Summary & Overview

This PR delivers a complete core rendering overhaul for Aether desktop widgets, per-widget configuration persistence, interactive quick-settings flyout controls on the WinUI 3 Dashboard, dynamic contrast legibility protection, automatic dependency process termination, and full automated test verification across both Rust backend daemon and C# WinUI 3 dashboard.

---

## ✨ New Features Added

### 🎨 1. Core Desktop Rendering & Widget Config Engine (Rust)
- **`WidgetConfigStore` (`crates/core_engine/src/widget_config_store.rs`)**:
  - Implemented per-widget JSON configuration persistence under `%LOCALAPPDATA%\Aether\widget_settings\<widget_id>.json`.
  - Persists `WidgetConfig`, `DisplayOptions` (`opacity`, `scale`, `locked`, `enabled`), `ColourOverrides`, and `quick_swap` options.
  - Full thread-safe concurrency with thread-safe `Arc<Mutex<WidgetConfigStore>>` integrated into `IpcSharedState`.
- **Expanded IPC Protocol (`crates/ipc_protocol/src/messages.rs` & `ipc_server.rs`)**:
  - Added new IPC control commands: `ListWidgets`, `UpdateWidgetDisplayOptions`, `QuickSwapWidget`, `EnableWidget`, `DisableWidget`, `SetWidgetOpacity`, `ResetWidgetConfig`.
  - Added `WidgetDescriptor` response payload for rich real-time widget state querying.
- **Dynamic Contrast Legibility Protection (`crates/widget_sdk/src/contrast.rs`)**:
  - Added `ContrastGuard::select_foreground_color(bg_argb, light, dark)` to dynamically select high-contrast foreground colors based on background sRGB relative luminance.
  - Applied contrast legibility protection to text and labels rendered in `perf_monitor_widget`.
- **Desktop Window Position Swap (`crates/core_engine/src/rendering/desktop_widget_window.rs`)**:
  - Added `swap_positions` method supporting real-time desktop coordinate swapping between any two widgets.

### ⚙️ 2. WinUI 3 Quick-Settings & Per-Widget Management (C#)
- **Per-Widget Quick Settings Flyout (`Pages/WidgetsPage.xaml`)**:
  - Added gear icon button (`⚙`) on each discovered and active running widget card.
  - Interactive Flyout UI featuring:
    - **Opacity Slider** (range `0.1`–`1.0` in steps of `0.05`) connected live via `SetOpacityCommand`.
    - **Enable/Disable Toggle** (`EnableWidget` / `DisableWidget`) via `ToggleEnableDisableCommand`.
    - **Drag-Lock Toggle** (`ToggleWidgetLockCommand`).
    - **Reset Settings Button** (`ResetWidgetConfigCommand`).
    - **Detailed Settings Button** opening an interactive `ContentDialog` showing manifest path, update interval, target FPS, and description.
- **`WidgetSettingsService` (`Services/WidgetSettingsService.cs`)**:
  - Manages reading/writing `%LOCALAPPDATA%\Aether\widget_settings\<widget_id>.json` and synchronizes live updates with the Rust Core Engine via Named Pipe IPC commands.
- **Enhanced `WidgetsViewModel` (`ViewModels/WidgetsViewModel.cs`)**:
  - Added `SetOpacityCommand`, `ToggleEnableDisableCommand`, `QuickSwapPositionCommand`, `QuickSwapConfigCommand`, and `ResetWidgetConfigCommand`.
  - Updated `DiscoverWidgetsAsync` and `RefreshRunningWidgets` to populate `WidgetInfo` with per-widget opacity, scale, locked, and enabled options from `WidgetSettingsService`.

### 🧹 3. Automatic Dependency Shutdown & Memory Management
- **Process Tree Cleanup**: `ProcessManagerService.StopEngineAsync()` kills background daemon processes with `entireProcessTree: true` on dashboard close.
- **Working-Set Memory Trimming**: `MemoryManagerService` uses Win32 `SetProcessWorkingSetSize` and forced garbage collection to return unused RAM back to Windows OS.

### 🧪 4. Automated Unit Test Suites
- **Rust Workspace Unit Tests**: 184 passing unit tests covering `widget_config_store`, `ipc_protocol` commands, `contrast.rs`, `perf_monitor_widget`, and subsystem orchestrators.
- **C# MSTest Unit Tests (`src_gui/CustomWidget.Dashboard.Tests`)**: 28 passing unit tests covering ViewModels, `WidgetSettingsService`, `MemoryManagerService`, `AetherIpcService`, and UI data models.

---

## 🛠️ Architecture & Design Governance

1. **Strict Core Rendering Boundary**:
   - Widgets are rendered exclusively on the desktop overlay layer (`HWND_BOTTOM`), click-through by default, positioned behind all applications. Widgets emit abstract `DrawCommand` lists; no widget spawns separate top-level UI windows.
2. **Interface Isolation**:
   - `WidgetConfigStore` operates through typed `WidgetConfig` abstractions without hardcoding concrete render target properties.
3. **No Premature Optimization**:
   - Per-widget JSON configurations use lightweight lazy disk persistence with atomic memory map caching.

---

## 📊 Verification & Test Metrics

| Metric | Result | Status |
|---|---|---|
| **Rust Workspace Tests** | **184 / 184 Passed** | ✅ PASS |
| **C# GUI Unit Tests** | **28 / 28 Passed** | ✅ PASS |
| **Total Test Suite Pass Rate** | **212 / 212 Passed (100%)** | ✅ PASS |
| **WinUI 3 Dashboard Build** | **0 Warnings, 0 Errors** | ✅ PASS |
| **Rust Engine Compilation** | **0 Errors** | ✅ PASS |

---

## 🔄 Comparison against Previous PR

| Feature / Area | Previous State (v0.6.0) | Current State (v0.7.0 PR) |
|---|---|---|
| **Widget Rendering** | Basic Desktop Window | **Core-Managed Desktop Rendering with Dynamic Contrast Legibility (`ContrastGuard`)** |
| **Widget Configuration** | System-wide config manager | **Per-Widget JSON Configs (`WidgetConfigStore`) + IPC Sync** |
| **Quick Settings UI** | Lock/Reset buttons only | **Per-Widget Quick Settings Flyout (Opacity Slider, Enable Toggle, Lock Toggle, Reset, Detailed Dialog)** |
| **Quick Swap** | None | **QuickSwap by Position Coordinates or Full Configuration** |
| **WinUI 3 Pages** | 5 Basic Pages | **13 Complete Pages** |
| **Automated Tests** | 121 Tests | **212 Tests (184 Rust + 28 C#)** |
