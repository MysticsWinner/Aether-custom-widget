# Terminal User Interface Dashboard (`dashboard_tui`)

**Purpose**: Documentation for the Ratatui-based terminal dashboard (`aether-dashboard`).  
**Audience**: Terminal Enthusiasts, System Administrators.  
**Prerequisites**: [IPC.md](../01_Architecture/IPC.md).  
**Related Documents**: [Dashboard.md](Dashboard.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / TUI App  
**Owner**: CLI & TUI Team  

---

## 1. Ratatui Gauges & Controls

Connects live to Named Pipe IPC (`\\.\pipe\CustomWidgetEngineControlPipe`) and renders animated CPU/GPU/RAM ASCII gauges, widget lists, and subsystem health status.

---

## Future Work
- Add keyboard shortcuts for inline widget configuration editing.

## Known Issues
- None.

## References
- [crates/dashboard_tui/src/main.rs](file:///d:/Code/Aether-custom-widget/crates/dashboard_tui/src/main.rs)

## Related Documents
- [Dashboard.md](Dashboard.md)
