# Startup & Shutdown Lifecycle

**Purpose**: Documents initialization sequence, subsystem startup order, and graceful reverse shutdown handling.  
**Audience**: Engine Engineers, System Integrators.  
**Prerequisites**: [System_Architecture.md](System_Architecture.md).  
**Related Documents**: [Engine.md](../02_Core/Engine.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Core Engine Team  

---

## 1. Sequential Startup Phase (< 45 ms)

1. Load configuration and initialize logging tracer.
2. Initialize `SharedTelemetryCache` and spawn background sensor sampler thread.
3. Register core subsystems with `SubsystemManager`.
4. Spawn Named Pipe IPC server.
5. Create desktop overlay window (`DesktopWidgetWindow`).

---

## 2. Reverse-Order Shutdown Phase

On receiving SIGINT/Ctrl+C or IPC `Shutdown` command, `SubsystemManager` iterates registered subsystems in reverse registration order, ensuring cleanup of resources before thread pool termination.

---

## Future Work
- Add fast crash dump flush handler during unhandled panic signals.

## Known Issues
- None.

## References
- [crates/core_engine/src/subsystems.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/subsystems.rs)

## Related Documents
- [Engine.md](../02_Core/Engine.md)
