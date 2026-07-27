# Aether SDK — Multi-Language Widget SDK Guide

## Overview

The **Aether SDK** provides standardized APIs across **Rust**, **C# .NET 8**, and **TypeScript** for creating desktop widgets.

---

## 6 Pillar SDK Architecture

1. **Lifecycle API**: Widget initialization, tick loop, pause/resume, and disposal hooks.
2. **Rendering API**: Flexbox UI layout definitions, text nodes, shapes, and image assets.
3. **Settings API**: Type-safe widget configuration storage (`widget.toml`).
4. **Events API**: Mouse interaction, keyboard shortcuts, and inter-widget messaging.
5. **Animations API**: Spring physics and easing curve properties.
6. **Resources API**: Hardware telemetry subscription streams (CPU, RAM, GPU, Network).

---

## Code Examples

### Rust Native Widget
```rust
use aether_sdk::prelude::*;

#[widget]
pub struct ClockWidget;

impl Widget for ClockWidget {
    fn on_tick(&mut self, ctx: &mut Context) {
        let time = ctx.system_time();
        ctx.render_text("time_label", &time.format("%H:%M:%S"));
    }
}
```

### C# .NET 8 Widget
```csharp
using Aether.SDK;

public class WeatherWidget : AetherWidget {
    public override void OnTick(WidgetContext ctx) {
        var temp = ctx.Telemetry.GetMetric("cpu.temp");
        ctx.UpdateElement("temp_display", $"{temp}°C");
    }
}
```

### TypeScript Widget
```typescript
import { AetherWidget, Context } from "@aether/sdk";

export class MemoryWidget extends AetherWidget {
    onTick(ctx: Context): void {
        const ramUsage = ctx.telemetry.get("memory.used_percent");
        ctx.setElementText("ram_val", `${ramUsage}%`);
    }
}
```
