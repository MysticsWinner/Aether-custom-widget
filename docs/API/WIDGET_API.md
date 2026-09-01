# Aether Widget SDK & Lifecycle API Reference (`widget_sdk`)

**Standardized 6-Pillar Widget API, Reactive Signal Bindings, and Batch Canvas Rendering**

---

## 1. The `WidgetLifecycle` Trait

Every native Aether widget plugin implements `WidgetLifecycle`:

```rust
use anyhow::Result;

pub trait WidgetLifecycle: Send + Sync {
    /// Allocate resources, load configuration, initialize state
    fn on_load(&mut self) -> Result<()>;

    /// Attach to desktop canvas window and mount DirectComposition visuals
    fn on_mount(&mut self) -> Result<()>;

    /// Per-tick lifecycle callback: read SharedTelemetryCache, update state, emit DrawCommands
    fn on_update(&mut self, ctx: &TickContext) -> Result<()>;

    /// Detach from desktop canvas
    fn on_unmount(&mut self) -> Result<()>;

    /// Free GPU resources, brushes, and caches
    fn on_unload(&mut self) -> Result<()>;

    /// Query current widget state machine state
    fn state(&self) -> WidgetState;
}
```

---

## 2. Rendering via `RenderCanvas` & `BatchRenderCanvas`

Widgets do not issue direct Win32/Direct2D API calls. Instead, they record drawing operations onto an abstract `RenderCanvas`:

```rust
pub trait RenderCanvas {
    fn draw_rect(&mut self, rect: RectF, color: Color, corner_radius: f32);
    fn draw_text(&mut self, text: &str, font: &str, size: f32, rect: RectF, color: Color);
    fn draw_gauge(&mut self, rect: RectF, value: f32, min: f32, max: f32, color: Color);
    fn commands(&self) -> &[DrawCommand];
}
```

Concrete implementation: `BatchRenderCanvas::new()` emits vector batches passed to the Direct2D compositing engine.

---

## 3. Reading Hardware Telemetry (`SharedTelemetryCache`)

Widgets read system telemetry exclusively from `SharedTelemetryCache`:

```rust
let snapshot = cache.get_snapshot();
let cpu_pct = snapshot.cpu_usage_pct;
let ram_used = snapshot.memory_used_mb;
let ram_total = snapshot.memory_total_mb;
let gpu_pct = snapshot.gpu_usage_pct;
```

Zero repeated kernel syscalls or context switches.

---

## 4. Reactive Signal Bindings (`Signal<T>`)

Widgets can subscribe to reactive metric signals using `SignalBinding`:
```rust
let mut cpu_signal = SignalBinding::new("sys.cpu_usage", 0.5 /* hysteresis threshold */);
if cpu_signal.poll(&cache) {
    // Redraw only when value delta exceeds threshold
}
```
