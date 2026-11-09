# macOS Platform Support Specification (Quartz / Metal)

**Purpose**: Architectural specifications for future macOS desktop layer support (Quartz / Metal desktop window levels).  
**Audience**: macOS Developers, Platform Engineers.  
**Prerequisites**: [ARCHITECTURE.md](../Architecture/ARCHITECTURE.md).  
**Related Documents**: [Windows.md](Windows.md), [Linux.md](Linux.md).  
**Last Updated**: 2026-08-07  
**Status**: Planned / Architectural Proposal  
**Owner**: Cross-Platform Working Group  

---

## 1. macOS Quartz / Metal Architecture Proposal

Attach transparent widget windows to `kCGDesktopWindowLevel` using `NSWindow` and render via Metal / `CAMetalLayer`.

---

## Future Work
- Expand `MacOsHostStatsProvider` (`crates/system_providers/src/providers/cross_platform.rs`) and `MacOsQuartzDesktopSurface` (`crates/core_engine/src/rendering/surface_trait.rs`) with live `objc2` / `Metal` window bindings.

## Known Issues
- Core trait (`DesktopSurface` at `kCGDesktopWindowLevel`) and Mach `vm_statistics64` + `host_cpu_load_info` calculators (`MacOsHostStatsProvider`) are implemented and unit-tested in v0.7.0.

## References
- [Detailed_Project_Report.md](../Project/Detailed_Project_Report.md)

## Related Documents
- [Windows.md](Windows.md)
