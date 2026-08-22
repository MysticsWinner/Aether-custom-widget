# Threading & Concurrency Model

**Purpose**: Explains thread allocation, Tokio async execution pool, and lock-free concurrency design in Aether.  
**Audience**: Core Engine Developers, Concurrency Engineers.  
**Prerequisites**: [System_Architecture.md](System_Architecture.md).  
**Related Documents**: [Memory_Model.md](Memory_Model.md), [Data_Flow.md](Data_Flow.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Core Engine Team  

---

## 1. Thread Pool Allocation

Aether divides execution across dedicated, isolated thread contexts:

1. **Tokio Async Worker Pool** (Default: $N$ Logical Cores):
   - Handles async Named Pipe IPC I/O, event bus broadcasting, and subsystem ticks.
2. **Dedicated Rendering Thread (Win32 Message Loop)**:
   - Owns native Win32 window handles (`HWND`) and GDI/DirectComposition composition targets. Prevents UI thread blocking.
3. **AppContainer Worker Process Pool**:
   - Out-of-process AppContainer worker processes executing sandboxed widget logic.

---

## 2. Lock-Free Synchronization

`SharedTelemetryCache` uses `Arc<RwLock<TelemetrySnapshot>>` and atomic sequence updates to ensure read operations from multiple widgets never block sensor ticks.

---

## Future Work
- Move GDI window message loop to worker thread pool via `PostThreadMessageW`.

## Known Issues
- None.

## References
- [crates/core_engine/src/rendering/desktop_widget_window.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/rendering/desktop_widget_window.rs)

## Related Documents
- [System_Architecture.md](System_Architecture.md)
- [Memory_Model.md](Memory_Model.md)
