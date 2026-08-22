# Hardware Rendering Architecture

**Purpose**: Explains Aether's DirectComposition, Direct2D, and GDI overlay hardware rendering architecture.  
**Audience**: Rendering Engineers, Graphics Developers.  
**Prerequisites**: [System_Architecture.md](System_Architecture.md).  
**Related Documents**: [DirectComposition.md](../03_Rendering/DirectComposition.md), [Direct2D.md](../03_Rendering/Direct2D.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Graphics Team  

---

## 1. Composition Architecture

Widgets emit high-level primitive batches (`DrawCommand::FillRect`, `DrawCommand::Text`). The rendering host composites primitives onto DWM desktop surfaces using:
- **`DirtyRegionTracker`**: Merges overlapping bounding boxes to skip unchanged screen areas.
- **`ContrastGuard`**: Evaluates WCAG 2.1 relative luminance and inverts text color if background transparency or luminance collisions threaten legibility.
- **`DisplayTarget`**: Pins widgets to `PrimaryMonitor`, `MonitorIndex(u32)`, or `AllMonitors`.

---

## Future Work
- Implement DirectComposition DX11 SwapChain presentation for zero-copy rendering.

## Known Issues
- None.

## References
- [crates/widget_sdk/src/rendering.rs](file:///d:/Code/Aether-custom-widget/crates/widget_sdk/src/rendering.rs)

## Related Documents
- [DirectComposition.md](../03_Rendering/DirectComposition.md)
