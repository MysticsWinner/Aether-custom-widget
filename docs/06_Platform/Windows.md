# Windows Platform Support Matrix (`Windows 11`)

**Purpose**: Win32, DirectComposition, WorkerW, and AppContainer API integration details for Windows 11 (`x86_64` & `ARM64`).  
**Audience**: Windows Developers, Platform Engineers.  
**Prerequisites**: [System_Architecture.md](../01_Architecture/System_Architecture.md).  
**Related Documents**: [WorkerW.md](../03_Rendering/WorkerW.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Primary Target  
**Owner**: Windows Platform Team  

---

## 1. Core Win32 Integration APIs

- **`windows 0.58` Crates**: Uses official `windows-rs` bindings for DirectComposition, Direct2D, DXGI, GDI, and System Information.
- **WorkerW Desktop Window Hook**: Message `0x052C` sent to Progman.
- **AppContainer Sandboxing**: Security SID isolation for 3rd-party widget processes.

---

## Future Work
- Support Windows 11 ARM64 native DirectComposition swapchains.

## Known Issues
- None.

## References
- [crates/core_engine/src/rendering/workerw.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/rendering/workerw.rs)

## Related Documents
- [WorkerW.md](../03_Rendering/WorkerW.md)
