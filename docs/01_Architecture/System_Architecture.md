# System Architecture

**Purpose**: Detailed technical design of Aether's core engine, subsystem orchestrator, and event loop.  
**Audience**: Core Engineers, System Architects.  
**Prerequisites**: [Architecture_Overview.md](../00_Project/Architecture_Overview.md).  
**Related Documents**: [Threading_Model.md](Threading_Model.md), [IPC.md](IPC.md), [Data_Flow.md](Data_Flow.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Core Engine Team  

---

## 1. Core Daemon Orchestrator

The core daemon (`crates/core_engine`) is an autonomous background host service written in Rust. It manages 9 core subsystem bridges via `SubsystemManager`:

```rust
pub trait Subsystem: Send + Sync {
    fn name(&self) -> &'static str;
    async fn initialize(&mut self, ctx: &SubsystemContext) -> Result<()>;
    async fn tick(&mut self, ctx: &TickContext) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    fn health(&self) -> SubsystemHealth;
}
```

---

## 2. Subsystem Registration Order

1. `TelemetrySubsystem` — Hardware sensor collection & cache warming.
2. `RenderSubsystem` — DirectComposition composition target initialization.
3. `ThemeEngineSubsystem` — Color token resolution & hot-reload watcher.
4. `PluginSandboxSubsystem` — AppContainer process supervisor.
5. `ProfilerSubsystem` — Microsecond frame time & memory profiler.
6. `MarketplaceSubsystem` — Registry solver & signature verifier.
7. `CloudSyncSubsystem` — CRDT state synchronization.
8. `AiSubsystem` — Natural language intent & layout synthesis.
9. `ProductionSubsystem` — Security auditor & auto-updater.

---

## Future Work
- Support dynamic hot-swapping of subsystems without daemon restart.

## Known Issues
- None.

## References
- [crates/core_engine/src/subsystems.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/subsystems.rs)

## Related Documents
- [Threading_Model.md](Threading_Model.md)
- [Data_Flow.md](Data_Flow.md)
