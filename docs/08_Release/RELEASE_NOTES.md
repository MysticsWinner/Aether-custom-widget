# Aether — Release Notes v0.5.0

**Official Release v0.5.0 (Production Release Candidate)**  
**Date**: August 2026  
**Repository**: [https://github.com/MysticsWinner/Aether-custom-widget](https://github.com/MysticsWinner/Aether-custom-widget)

---

## 🚀 What's New in Version 0.5.0

Aether v0.5.0 represents a major milestone in the evolution of the next-generation Windows desktop customization platform. This release connects real Win32 kernel hardware APIs to an asynchronous Tokio engine daemon, DirectComposition transparent window overlays, a C# WinUI 3 management dashboard, andRatatui TUI dashboard.

### 🌟 Key Highlights & Features

1. **Task Manager-Style Telemetry & EMA Smoothing**:
   - Integrated native Windows APIs (`GetSystemTimes`, `GlobalMemoryStatusEx`, `IDXGIFactory1::QueryVideoMemoryInfo`, `GetIfTable2`).
   - Exponential Moving Average (EMA, $\alpha = 0.25$) temporal smoothing across CPU, GPU, RAM, and Network metrics.
   - Sub-quantum zero-delta tick holding for rock-solid CPU percentage stability.

2. **Interactive Drag-to-Position & Lock Store**:
   - Native Win32 `WM_NCHITTEST` hit testing on transparent desktop overlay cards, allowing smooth mouse drag-and-drop repositioning.
   - Lock/Unlock position controls (`is_locked: bool`) to prevent accidental mouse movements.
   - Automatic non-destructive layout persistence saved to `.aether/widget_positions.json`.

3. **WinUI 3 Management Dashboard Enhancements**:
   - Live visual theme switching (`Dark`, `Light`, `System`) via `ElementTheme` sync + IPC command `SetThemeModeAsync`.
   - Dedicated **About Page** featuring GitHub repository links, team & contributor credits, architecture specs, and release metadata.
   - Interactive widget cards with Lock/Unlock position toggles and Reset Position controls.

4. **Lua Scripting Host Engine Expansion**:
   - Embedded `mlua` 5.4 host environment with telemetry SDK queries (`get_cpu_pct`, `get_gpu_pct`, `get_memory_mb`, `get_net_rate`) and position/lock query functions (`get_widget_position`, `is_widget_locked`).

5. **Dynamic System Accent & AI Synthesis**:
   - Real-time Windows 11 system accent color polling (`DwmGetColorizationColor`) updating `theme.accent` tokens.
   - AI layout and widget manifest schema generator (`WidgetGenerator`) synthesizing declarative TOML specifications.

6. **CRDT Cloud Sync & Package Manager**:
   - Vector clock & Last-Write-Wins (`LwwResolver`) CRDT layout position synchronization.
   - Ed25519 cryptographic package signature verifier for `.aetherpkg` marketplace archives.

---

## 🛠️ Verification & Test Metrics
- **Crate Test Suite**: **107/107 tests passing** (100% pass rate).
- **Workspace Build**: Zero compilation warnings / zero errors on Windows 11 (`x86_64` & `ARM64`).
