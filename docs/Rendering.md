# Aether Renderer — DirectComposition GPU Compositing Pipeline

## Overview

**Aether Renderer** bypasses traditional Win32 GDI/GDI+ software rendering by composing 2D visual trees directly onto Windows Desktop Windows Manager (DWM) compositor surfaces (`WorkerW`).

---

## Rendering Pipeline Stages

1. **Flexbox Layout Solving (`layout_engine`)**: Evaluates `taffy` flexbox properties to determine absolute bounding boxes.
2. **Dirty Region Tracking (`DirtyRegionTracker`)**: Identifies visual elements modified during the current tick (`PushAxisAlignedClip`).
3. **DirectWrite & Direct2D Rasterization**: Vector primitives and text strings are rasterized using Direct2D 1.1 hardware contexts.
4. **DirectComposition Tree Swap**: Visual elements are composited directly into the Windows DWM surface tree at up to 144Hz+.

---

## Performance Targets
- Frame Render Time: **`< 0.5 ms`**
- Redraw Culling Efficiency: **`92.4%`**
- VRAM Footprint: **`< 12 MB`**
