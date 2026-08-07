# Widget Runtime Subsystem (`widget_sdk`)

**Purpose**: Widget lifecycle state machine, event dispatch, and batch primitive execution.  
**Audience**: Widget Developers, Core Engineers.  
**Prerequisites**: [Widget_SDK.md](../04_SDK/Widget_SDK.md).  
**Related Documents**: [Plugin_Runtime.md](Plugin_Runtime.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: SDK & Widget Team  

---

## 1. Widget State Lifecycle

Transitions through `Unloaded` → `Loaded` → `Mounted` → `Unmounted` → `Unloaded`.

```rust
pub trait WidgetLifecycle {
    fn on_load(&mut self) -> Result<()>;
    fn on_mount(&mut self) -> Result<()>;
    fn on_update(&mut self, ctx: &TickContext) -> Result<()>;
    fn on_unmount(&mut self) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
}
```

---

## Future Work
- Add async lifecycle hook methods (`on_load_async`).

## Known Issues
- None.

## References
- [crates/widget_sdk/src/lifecycle.rs](file:///d:/Code/Aether-custom-widget/crates/widget_sdk/src/lifecycle.rs)

## Related Documents
- [Widget_SDK.md](../04_SDK/Widget_SDK.md)
