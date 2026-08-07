# Event System Architecture

**Purpose**: Technical documentation of Aether's central broadcast event bus (`CoreEvent`).  
**Audience**: Engine Developers, Plugin Developers.  
**Prerequisites**: [System_Architecture.md](System_Architecture.md).  
**Related Documents**: [Data_Flow.md](Data_Flow.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Core Engine Team  

---

## 1. CoreEvent Enum

Subsystems and widgets communicate asynchronously via `EventBus` (`tokio::sync::broadcast` channel):

```rust
pub enum CoreEvent {
    TelemetryTick(TelemetrySnapshot),
    ThemeChanged(String),
    WidgetLoaded { widget_id: String },
    WidgetUnloaded { widget_id: String },
    SystemStateChanged(SystemState),
}
```

---

## Future Work
- Add high-priority event filter channels to bypass lower-priority telemetry ticks.

## Known Issues
- None.

## References
- [crates/core_engine/src/event_bus.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/event_bus.rs)

## Related Documents
- [System_Architecture.md](System_Architecture.md)
