# WinUI 3 Desktop Dashboard (`CustomWidget.Dashboard`)

**Purpose**: Documentation for the C# WinUI 3 management GUI dashboard app.  
**Audience**: Application Developers, UI Designers, Users.  
**Prerequisites**: [IPC.md](../01_Architecture/IPC.md).  
**Related Documents**: [TUI.md](TUI.md), [Settings.md](Settings.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / GUI App  
**Owner**: Desktop Application Team  

---

## 1. Features & Capabilities

- **Live Telemetry Gauges**: Real-time CPU, GPU, RAM, Network, Volume, and Battery meters.
- **Widget Manager**: One-click Load, Unload, Lock, and Position tuning.
- **Marketplace Hub**: Search, filter, and install cryptographically signed widget packages (`.cwp`) with Ed25519 signature validation.
- **Snapshots & Recovery Hub**: Create, list, restore, export, and delete transactional system configuration snapshots.
- **Security & Sandbox Visualizer**: Real-time AppContainer process boundary monitor, active capability tokens, Job Object resource limits, and audit logs.
- **Render Config Panel**: Opacity sliders, glass blur selection, background color picker, display target pinning.
- **Log Viewer**: Integrated log file browser for `logs/engine.log` and `logs/dashboard.log`.

---

## 2. Build & Test Verification

Every code modification affecting the GUI application must be verified using both Rust and C# test protocols:

```powershell
# 1. Verify C# WinUI 3 Dashboard build
dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj

# 2. Run C# GUI MSTest Unit Test Suite
dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj

# 3. Run Rust Core Engine tests & check
cargo check --workspace
cargo test --workspace
```

---

## Known Issues
- Requires Visual Studio 2022 and Windows App SDK 1.5 to build.

## References
- [src_gui/CustomWidget.Dashboard/](file:///d:/Code/Aether-custom-widget/src_gui/CustomWidget.Dashboard)

## Related Documents
- [IPC.md](../01_Architecture/IPC.md)
