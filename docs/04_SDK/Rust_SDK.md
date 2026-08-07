# Native Rust Widget SDK (`widget_sdk`)

**Purpose**: API reference and code samples for building high-performance native Rust widgets.  
**Audience**: Rust Developers.  
**Prerequisites**: [Widget_SDK.md](Widget_SDK.md).  
**Related Documents**: [Plugin_API.md](Plugin_API.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / SDK Guide  
**Owner**: SDK Team  

---

## 1. Minimal Rust Widget Example

```rust
use widget_sdk::{WidgetLifecycle, TickContext, BatchRenderCanvas, Color, RectF, Result};

pub struct MyWidget;

impl WidgetLifecycle for MyWidget {
    fn on_load(&mut self) -> Result<()> { Ok(()) }
    fn on_mount(&mut self) -> Result<()> { Ok(()) }
    fn on_update(&mut self, ctx: &TickContext) -> Result<()> {
        let mut canvas = BatchRenderCanvas::new();
        canvas.draw_rect(RectF { x: 0.0, y: 0.0, w: 200.0, h: 100.0 }, Color::rgba(15, 23, 42, 216), 12.0);
        canvas.draw_text("Aether Native Widget", "Segoe UI", 14.0, RectF { x: 16.0, y: 16.0, w: 168.0, h: 20.0 }, Color::rgb(0, 212, 245));
        Ok(())
    }
    fn on_unmount(&mut self) -> Result<()> { Ok(()) }
    fn on_unload(&mut self) -> Result<()> { Ok(()) }
}
```

---

## Future Work
- Add `#[derive(AetherWidget)]` procedural macro helper.

## Known Issues
- None.

## References
- [crates/perf_monitor_widget/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/perf_monitor_widget/src/lib.rs)

## Related Documents
- [Widget_SDK.md](Widget_SDK.md)
