# Changelog

**Purpose**: Chronological log of version changes, features, bug fixes, and security enhancements in Aether.  
**Audience**: All Developers, Maintainers, Users.  
**Prerequisites**: [Release_Notes.md](Release_Notes.md).  
**Related Documents**: [Project_Status.md](Project_Status.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Version Log  
**Owner**: Release Engineering Lead  

---

## [v0.7.0-rc.3] - 2026-08-07

### Added
- **Plan 0.7 Phases 17–25**:
  - `crates/recovery_manager/`: Crash loop detection, widget quarantine, Safe Mode sentinel.
  - `crates/config_manager/`: Atomic transactional writes, 5-gen backup rotation, desktop snapshot manager.
  - `crates/capability_broker/`: Sandboxing permission broker, `WidgetFirewall`, BLAKE3 binary integrity.
  - `crates/watchdog/` & `crates/event_recorder/`: Two-process heartbeat supervisor & time-travel event recording.
  - `crates/observability/`: Prometheus exporter (`/metrics`), Win32 crash minidump writer (`.dmp`), ETW provider.
  - `crates/system_providers/` & `crates/widget_sdk/`: `TickRateAdvisor` (10ms-100ms adaptive tick), `FrameScheduler`, `LruResourceCache`.
  - `crates/dev_tools/`: File-watcher hot-reloader, Chrome-style DOM inspector, layout grid overlay.
  - `crates/ai_engine/` & `crates/package_manager/`: `WidgetSynthesizer`, `WallpaperThemeGenerator`, `AiPerformanceAdvisor`, `MarketplaceCatalog`.
  - `crates/enterprise/`: Group Policy engine, SHA-256 tamper-evident audit logger, Windows Hello biometric gate.
  - `crates/widget_sdk/`: `ContrastGuard` (WCAG 2.1 contrast), `DisplayTarget` (multi-monitor), `RenderConfig`.
- **Extended Telemetry**: Process counts (open apps, browser tabs, audio apps, gaming apps, dev apps), power/battery metrics (charge %, remaining secs, charging state), master volume %, multi-GPU & display topology.
- **Documentation Overhaul**: Scalable 10-domain documentation architecture with `Detailed_Project_Report.md` master SSOT encyclopedia.

---

## [v0.6.0] - 2026-08-06

### Added
- WinUI 3 C# Desktop Management Dashboard (`CustomWidget.Dashboard`).
- Named Pipe IPC async server (`\\.\pipe\CustomWidgetEngineControlPipe`).
- Ratatui TUI dashboard (`dashboard_tui`).
- Comprehensive 116-test suite passing cleanly.

---

## Future Work
- Release v0.7.0 Production Release.

## Known Issues
- None.

## References
- [Release_Notes.md](Release_Notes.md)

## Related Documents
- [Project_Status.md](Project_Status.md)
