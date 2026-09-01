# Aether Widget SDK & Lifecycle API Reference (`widget_sdk`)

**Standardized 6-Pillar Widget API, Reactive Signal Bindings, Interactive Hit-Testing, and Batch Canvas Rendering**

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

Widgets record drawing operations onto an abstract `RenderCanvas`:

```rust
pub trait RenderCanvas {
    fn clear(&mut self, color: Color);
    fn draw_rect(&mut self, rect: RectF, color: Color, corner_radius: f32);
    fn draw_text(&mut self, text: &str, font_family: &str, font_size: f32, rect: RectF, color: Color);
    fn draw_image(&mut self, resource_id: &str, rect: RectF, opacity: f32);
    fn draw_path(&mut self, path_data: &str, fill_color: Option<Color>, stroke_color: Option<Color>, stroke_width: f32);
    fn draw_spline_area(&mut self, points: Vec<(f32, f32)>, fill_color: Color, line_color: Color, line_width: f32);
    fn draw_effect(&mut self, effect: RenderEffect, rect: RectF);
    fn push_clip(&mut self, rect: RectF);
    fn pop_clip(&mut self);
    fn invalidate(&mut self, rect: RectF);
    fn commands(&self) -> &[DrawCommand];
}
```

---

## 3. Interactive Hit-Testing & Gesture Pipeline

Widgets can register bounding boxes for pointer interaction:

```rust
let mut hit_tree = HitTestTree::new();
hit_tree.register_target(HitTarget::new("button_refresh", RectF::new(200.0, 20.0, 80.0, 30.0)));

if let Some(target) = hit_tree.hit_test(cursor_x, cursor_y) {
    // Dispatch click/hover event to target element
}
```

---

## 4. Reactive Signals & Computed Formulas

```rust
let mut cpu_signal = Signal::new("sys.cpu_usage", 45.0f32);
let mut is_high_load = Computed::new("high_load", || *cpu_signal.get() >= 80.0);
```

---

## 5. Reading Hardware Telemetry (`SharedTelemetryCache`)

Widgets read system telemetry exclusively from `SharedTelemetryCache`:

```rust
let snapshot = cache.get_snapshot();
let cpu_pct = snapshot.cpu_usage_pct;
let vram_used = snapshot.gpu_telemetry.vram_dedicated_used_mb;
let fft_bins = snapshot.audio_spectrum.fft_bins_16;
let song_title = snapshot.media_playback.title;
```

---

## References
- [crates/widget_sdk/src/lifecycle.rs](../../crates/widget_sdk/src/lifecycle.rs)
- [crates/widget_sdk/src/rendering.rs](../../crates/widget_sdk/src/rendering.rs)
- [crates/widget_sdk/src/events.rs](../../crates/widget_sdk/src/events.rs)
- [crates/widget_sdk/src/reactive.rs](../../crates/widget_sdk/src/reactive.rs)
- [crates/widget_sdk/src/svg_parser.rs](../../crates/widget_sdk/src/svg_parser.rs)
