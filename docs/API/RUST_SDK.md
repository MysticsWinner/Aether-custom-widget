# Native Rust Widget SDK (`widget_sdk`)

**6-Pillar Rust Widget Framework, Lifecycle Hooks, Batch Canvas, and Spring Animations**

---

## 1. Quickstart: Creating a Rust Widget Plugin

### `Cargo.toml`
```toml
[package]
name = "my_custom_widget"
version = "0.1.0"
edition = "2021"

[dependencies]
widget_sdk = { path = "../../crates/widget_sdk" }
system_providers = { path = "../../crates/system_providers" }
anyhow = "1.0"
```

### `src/lib.rs`
```rust
use anyhow::Result;
use widget_sdk::lifecycle::{WidgetLifecycle, WidgetState, TickContext};
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas};

pub struct MyCustomWidget {
    state: WidgetState,
    canvas: BatchRenderCanvas,
}

impl MyCustomWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::Unloaded,
            canvas: BatchRenderCanvas::new(),
        }
    }
}

impl WidgetLifecycle for MyCustomWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        Ok(())
    }

    fn on_update(&mut self, ctx: &TickContext) -> Result<()> {
        self.canvas.clear();
        let snap = ctx.cache.get_snapshot();
        self.canvas.draw_rect(RectF::new(0.0, 0.0, 200.0, 100.0), Color::rgba(20, 24, 35, 220), 12.0);
        self.canvas.draw_text(&format!("CPU: {:.1}%", snap.cpu_usage_pct), "Segoe UI", 14.0, RectF::new(16.0, 16.0, 168.0, 24.0), Color::rgb(0, 240, 255));
        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        Ok(())
    }

    fn state(&self) -> WidgetState {
        self.state
    }
}
```
