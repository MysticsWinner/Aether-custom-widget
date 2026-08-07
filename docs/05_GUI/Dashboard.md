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
- **Render Config Panel**: Opacity sliders, glass blur selection, background color picker, display target pinning.
- **Log Viewer**: Integrated log file browser for `logs/engine.log` and `logs/dashboard.log`.

---

## Future Work
- Add drag-and-drop visual layout editor canvas.

## Known Issues
- Requires Visual Studio 2022 and Windows App SDK 1.5 to build.

## References
- [src_gui/CustomWidget.Dashboard/](file:///d:/Code/Aether-custom-widget/src_gui/CustomWidget.Dashboard)

## Related Documents
- [IPC.md](../01_Architecture/IPC.md)
