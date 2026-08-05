# Aether — Rust Widget SDK Reference (`widget_sdk`)

**Lifecycle Traits, Rendering Canvas, and Event Subscriptions**

---

## 1. `WidgetLifecycle` Trait Reference

Every native Rust widget plugin implements the `WidgetLifecycle` trait in `widget_sdk::lifecycle`:

```rust
pub trait WidgetLifecycle: Send + Sync {
    /// Initial resource allocation
    fn on_load(&mut self) -> Result<()> { Ok(()) }

    /// Attach widget visual surface to desktop
    fn on_mount(&mut self) -> Result<()> { Ok(()) }

    /// Invoked per tick cycle to read telemetry and record draw commands
    fn on_update(&mut self, ctx: &TickContext) -> Result<()> { Ok(()) }

    /// Detach widget visual surface from desktop
    fn on_unmount(&mut self) -> Result<()> { Ok(()) }

    /// Free allocated resources
    fn on_unload(&mut self) -> Result<()> { Ok(()) }

    /// Query current lifecycle state
    fn state(&self) -> WidgetState;
}
```

---

## 2. `RenderCanvas` & `BatchRenderCanvas` API

Widgets emit drawing commands into a `BatchRenderCanvas` during `on_update()`:

```rust
let mut canvas = BatchRenderCanvas::new();

// Draw glassmorphism dark card background
canvas.draw_rect(
    RectF { x: 0.0, y: 0.0, width: 300.0, height: 150.0 },
    Color { r: 0.05, g: 0.05, b: 0.08, a: 0.85 },
    12.0, // Corner radius
);

// Draw CPU metric text
canvas.draw_text(
    &format!("CPU Usage: {:.1}%", cpu_pct),
    "Segoe UI",
    14.0,
    RectF { x: 16.0, y: 16.0, width: 200.0, height: 24.0 },
    Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
);

// Retrieve batched commands
let commands: &[DrawCommand] = canvas.commands();
```

---

## 3. Widget Settings Store (`widget_sdk::settings`)

Key-value settings persistence using `SettingsStore`:

```rust
let mut store = SettingsStore::new();
store.set("refresh_interval_ms", serde_json::json!(500));
let interval = store.get_u64("refresh_interval_ms").unwrap_or(1000);
```
