# Diagnostics & Live Profiler (`dev_tools`)

**Purpose**: Guide to using Chrome-style DOM inspector, layout grid overlay, and ETW profiling.  
**Audience**: Developers, Performance Engineers.  
**Prerequisites**: [Dashboard.md](Dashboard.md).  
**Related Documents**: [Benchmarks.md](../08_Testing/Benchmarks.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Diagnostic Tool  
**Owner**: Dev Tools Team  

---

## 1. Dev Tools Diagnostics Panel

- **`InspectWidget` IPC Command**: Returns microsecond frame execution breakdown and memory allocations.
- **`ToggleLayoutGrid` IPC Command**: Draws aligned 8px grid overlay over desktop window for visual debugging.

---

## Future Work
- Add remote web browser WebSocket debugging port.

## Known Issues
- None.

## References
- [crates/dev_tools/src/inspector.rs](file:///d:/Code/Aether-custom-widget/crates/dev_tools/src/inspector.rs)

## Related Documents
- [Dashboard.md](Dashboard.md)
