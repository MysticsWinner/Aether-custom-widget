# Linux Platform Support Specification (Wayland & X11)

**Purpose**: Architectural specifications for future Linux desktop layer support (Wayland layer-shell protocol / X11 root window).  
**Audience**: Linux Platform Engineers, Open-Source Contributors.  
**Prerequisites**: [ARCHITECTURE.md](../Architecture/ARCHITECTURE.md).  
**Related Documents**: [Windows.md](Windows.md), [macOS.md](macOS.md).  
**Last Updated**: 2026-08-07  
**Status**: Planned / Architectural Proposal  
**Owner**: Cross-Platform Working Group  

---

## 1. Wayland & X11 Architecture Proposal

- **Wayland**: Use `wlr-layer-shell-unstable-v1` protocol to attach widget surfaces to the `ZWLR_LAYER_SHELL_V1_LAYER_BACKGROUND` desktop layer.
- **X11**: Render to `_NET_WM_WINDOW_TYPE_DESKTOP` root window.

---

## Future Work
- Expand `LinuxProcfsProvider` (`crates/system_providers/src/providers/cross_platform.rs`) and `LinuxWaylandLayerSurface` (`crates/core_engine/src/rendering/surface_trait.rs`) with live `wayland-client` socket bindings.

## Known Issues
- Core trait (`DesktopSurface`) and `/proc/stat` + `/proc/meminfo` parsers (`LinuxProcfsProvider`) are implemented and unit-tested in v0.7.0.

## References
- [Detailed_Project_Report.md](../Project/Detailed_Project_Report.md)

## Related Documents
- [Windows.md](Windows.md)
