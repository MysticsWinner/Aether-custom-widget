# Technical Debt

*Code quality issues, shortcuts, and deferred cleanup items.*

---

## TD-001: GDI fallback in `desktop_widget_window.rs` needs DX11 SwapChain migration

- **Severity**: Medium
- **Component**: `core_engine` — `crates/core_engine/src/rendering/desktop_widget_window.rs`
- **Impact**: The GDI `UpdateLayeredWindow` fallback path uses software blitting instead of DirectComposition hardware compositing, resulting in higher CPU usage and no vsync.
- **Status**: Open (existing, carried forward)

## TD-002: `dashboard_tui` has dead code warnings on unused status response fields

- **Severity**: Low
- **Component**: `dashboard_tui`
- **Impact**: Compiler warnings in the TUI crate.
- **Status**: Open (existing, carried forward)

## TD-003: Hardcoded mock/fallback values in GPU, Audio, and Process providers

- **Severity**: Medium
- **Component**: `system_providers` — `GpuProvider`, `AudioProvider`, `ProcessMetricsProvider`
- **Impact**: Several providers return hardcoded or heuristically estimated values when real Win32 API calls fail or are unimplemented. See also BUG-001 and BUG-002.
- **Status**: Open (existing, carried forward with new detail)

## TD-004: C# Named Pipe sync call blocks WinUI 3 UI thread on startup

- **Severity**: Medium
- **Component**: `CustomWidget.Dashboard` — IPC client initialization
- **Impact**: Brief UI freeze during initial pipe connection.
- **Status**: Open (existing, carried forward)

## TD-005: DirectComposition WorkerW unhook on display resolution change

- **Severity**: High
- **Component**: `core_engine` — `crates/core_engine/src/rendering/workerw.rs`
- **Impact**: Desktop overlay may detach or render incorrectly when display resolution changes or monitors are hot-plugged.
- **Status**: Open (existing, carried forward)

## TD-006: `WidgetState` enum has 13 variants but docs claim 11

- **Severity**: Low
- **Component**: `widget_sdk` — `crates/widget_sdk/src/lifecycle.rs`, `docs/Architecture/ARCHITECTURE.md`
- **Impact**: Documentation claims 11 widget states but the actual `WidgetState` enum has 13 variants (includes `Mounted` and `Unmounted` as backward-compatibility aliases for `Active` and after-unmount). This creates confusion for contributors.
- **Status**: Open — documented in this overhaul.

## TD-007: Multiple widget crates have unused field warnings

- **Severity**: Low
- **Component**: `dock_launcher_widget`, `audio_visualizer_widget`, `hardware_pro_widget`, `crypto_stocks_widget`, `weather_particles_widget`
- **Impact**: `cargo check` emits warnings about `material`, `budget`, and `config` fields that are defined in widget structs but never read. These fields were likely added as placeholders for future design-token and performance-budget integration.
- **Status**: Open
