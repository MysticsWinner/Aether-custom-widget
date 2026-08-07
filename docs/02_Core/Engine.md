# Engine Subsystem (`core_engine`)

**Purpose**: Technical guide for Aether's core engine orchestrator, tick loop, and subsystem manager.  
**Audience**: Engine Developers.  
**Prerequisites**: [System_Architecture.md](../01_Architecture/System_Architecture.md).  
**Related Documents**: [Scheduler.md](Scheduler.md), [Telemetry.md](Telemetry.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: Core Engine Team  

---

## 1. Engine Lifecycle State Machine

```rust
pub enum EngineState {
    Created,
    Running,
    Paused,
    Stopped,
}
```

The engine orchestrator manages state transitions and dispatches ticks across all registered subsystems.

---

## Future Work
- Add dynamic thread affinity pinning for engine tick loop.

## Known Issues
- None.

## References
- [crates/core_engine/src/engine.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/engine.rs)

## Related Documents
- [Scheduler.md](Scheduler.md)
