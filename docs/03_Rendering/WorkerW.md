# WorkerW Desktop Hook Subsystem (`workerw.rs`)

**Purpose**: Technical details of Win32 Progman `0x052C` message hook attaching transparent widget windows behind desktop icons.  
**Audience**: Windows System Engineers, Graphics Developers.  
**Prerequisites**: [Rendering.md](../01_Architecture/Rendering.md).  
**Related Documents**: [Windows.md](../06_Platform/Windows.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Platform Hook  
**Owner**: Windows Platform Team  

---

## 1. Win32 WorkerW Hook Mechanism

Sends undocumented Win32 message `0x052C` to Windows `Progman` window handle to spawn a `WorkerW` window host between desktop icons and wallpaper. `SetParent(hwnd, workerw_hwnd)` attaches Aether widget windows cleanly behind desktop icons.

---

## Future Work
- Add auto-rehook handler when Windows Explorer (`explorer.exe`) restarts.

## Known Issues
- None.

## References
- [crates/core_engine/src/rendering/workerw.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/rendering/workerw.rs)

## Related Documents
- [Windows.md](../06_Platform/Windows.md)
