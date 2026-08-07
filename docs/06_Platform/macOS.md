# macOS Platform Support Specification (Quartz / Metal)

**Purpose**: Architectural specifications for future macOS desktop layer support (Quartz / Metal desktop window levels).  
**Audience**: macOS Developers, Platform Engineers.  
**Prerequisites**: [System_Architecture.md](../01_Architecture/System_Architecture.md).  
**Related Documents**: [Windows.md](Windows.md), [Linux.md](Linux.md).  
**Last Updated**: 2026-08-07  
**Status**: Planned / Architectural Proposal  
**Owner**: Cross-Platform Working Group  

---

## 1. macOS Quartz / Metal Architecture Proposal

Attach transparent widget windows to `kCGDesktopWindowLevel` using `NSWindow` and render via Metal / `CAMetalLayer`.

---

## Future Work
- Implement `macos_providers` crate using `host_statistics` and `IOKit` hardware samplers.

## Known Issues
- Not currently implemented in v0.7.0.

## References
- [Detailed_Project_Report.md](../00_Project/Detailed_Project_Report.md)

## Related Documents
- [Windows.md](Windows.md)
