# Widget SDK Developer Guide

Welcome to the **Multi-Language Widget SDK** for the Next-Gen Windows Desktop Customization Platform. Developers can author high-performance, hardware-accelerated desktop widgets in **Rust**, **C#**, or **TypeScript**.

---

## 🌟 The 6 Core SDK API Pillars

```
+-----------------------------------------------------------------------------------+
|                                 6 CORE SDK PILLARS                                |
+------------------+------------------+------------------+------------------+------------------+------------------+
| 1. Lifecycle     | 2. Rendering     | 3. Settings      | 4. Events        | 5. Animations    | 6. Resources     |
| on_load          | clear            | get              | on_event         | SpringAnimation  | load_asset       |
| on_mount         | draw_rect        | set              | InputEvent       | EasingCurve      | resolve_token    |
| on_update        | draw_text        | contains         | Telemetry        | keyframes        | font_cache       |
| on_unmount       | draw_image       |                  | ThemeChanged     |                  |                  |
| on_unload        | push_clip        |                  |                  |                  |                  |
+------------------+------------------+------------------+------------------+------------------+------------------+
```

---

## 🦀 1. Rust Native Widget SDK (`crates/widget_sdk`)

### Cargo Dependency
```toml
[dependencies]
widget_sdk = { path = "../widget_sdk" }
```

### Rust Example Implementation
```rust
use widget_sdk::{
    Color, DrawCommand, InMemorySettingsStore, RectF, RenderCanvas, SettingValue,
    SettingsStore, SpringAnimation, SpringParams, TickContext, WidgetLifecycle, WidgetState,
};

pub struct SystemMonitorWidget {
    state: WidgetState,
    cpu_animation: SpringAnimation,
    settings: InMemorySettingsStore,
}

impl SystemMonitorWidget {
    pub fn new() -> Self {
        Self {
            state: WidgetState::Unloaded,
            cpu_animation: SpringAnimation::new(0.0, 0.0, SpringParams::default()),
            settings: InMemorySettingsStore::new(),
        }
    }
}

impl WidgetLifecycle for SystemMonitorWidget {
    fn on_load(&mut self) -> anyhow::Result<()> {
        self.state = WidgetState::Loaded;
        self.settings.set("refresh_ms", SettingValue::Integer(1000))?;
        Ok(())
    }

    fn on_mount(&mut self) -> anyhow::Result<()> {
        self.state = WidgetState::Mounted;
        Ok(())
    }

    fn on_update(&mut self, ctx: &TickContext) -> anyhow::Result<()> {
        let dt_sec = ctx.delta_time_ms / 1000.0;
        self.cpu_animation.update(dt_sec);
        Ok(())
    }

    fn on_unmount(&mut self) -> anyhow::Result<()> {
        self.state = WidgetState::Unmounted;
        Ok(())
    }

    fn state(&self) -> WidgetState {
        self.state
    }
}
```

---

## ⚡ 2. C# / .NET 8 Widget SDK (`bindings/csharp/CustomWidget.SDK`)

### C# Widget Interface
```csharp
using System.Threading.Tasks;
using CustomWidget.SDK;

namespace MyCustomWidgets
{
    public class WeatherWidget : IWidget
    {
        public WidgetState State { get; private set; } = WidgetState.Unloaded;

        public Task OnLoadAsync()
        {
            State = WidgetState.Loaded;
            return Task.CompletedTask;
        }

        public Task OnMountAsync()
        {
            State = WidgetState.Mounted;
            return Task.CompletedTask;
        }

        public void OnUpdate(in TickContext context, IRenderCanvas canvas)
        {
            // Clear & Draw Background Card
            canvas.Clear(new Color { R = 0.1f, G = 0.1f, B = 0.1f, A = 0.8f });
            canvas.DrawRect(new RectF { X = 10, Y = 10, Width = 280, Height = 120 },
                            new Color { R = 0.0f, G = 0.47f, B = 0.84f, A = 1.0f },
                            cornerRadius: 8.0f);

            // Render Weather Typography
            canvas.DrawText("72°F Sunny", "Segoe UI", 24.0f,
                            new RectF { X = 20, Y = 20, Width = 200, Height = 40 },
                            new Color { R = 1.0f, G = 1.0f, B = 1.0f, A = 1.0f });
        }

        public Task OnUnmountAsync()
        {
            State = WidgetState.Unmounted;
            return Task.CompletedTask;
        }

        public Task OnUnloadAsync()
        {
            State = WidgetState.Unloaded;
            return Task.CompletedTask;
        }

        public void OnEvent(string topic, string payload) { }
    }
}
```

---

## 🔷 3. TypeScript Widget SDK (`bindings/typescript/custom-widget-sdk`)

### TypeScript Definition & Widget Class
```typescript
import { Widget, WidgetState, TickContext, RenderCanvas, Color, RectF } from 'custom-widget-sdk';

export class CpuGaugeWidget implements Widget {
  public state: WidgetState = 'Unloaded';

  public onLoad(): void {
    this.state = 'Loaded';
  }

  public onMount(): void {
    this.state = 'Mounted';
  }

  public onUpdate(ctx: TickContext, canvas: RenderCanvas): void {
    const bg: Color = { r: 0.1, g: 0.1, b: 0.1, a: 0.9 };
    const accent: Color = { r: 0.0, g: 0.47, b: 0.84, a: 1.0 };
    const textCol: Color = { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    canvas.clear(bg);
    canvas.drawRect({ x: 0, y: 0, width: 300, height: 150 }, accent, 8);
    canvas.drawText('CPU Utilization: 45%', 'Segoe UI', 16, { x: 15, y: 15, width: 250, height: 30 }, textCol);
  }

  public onUnmount(): void {
    this.state = 'Unmounted';
  }
}
```
