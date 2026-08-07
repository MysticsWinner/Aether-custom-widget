# GPU Pipeline Architecture

**Purpose**: High-level specification of DirectX 11/12 GPU rendering pipeline.  
**Audience**: Graphics Engineers.  
**Prerequisites**: [Rendering.md](../01_Architecture/Rendering.md).  
**Related Documents**: [DirectComposition.md](DirectComposition.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Graphics Team  

---

## 1. Pipeline Stages

1. Primitive Extraction (`BatchRenderCanvas`).
2. Retained Dirty Region Culling (`DirtyRegionTracker`).
3. Direct2D Command List Recording.
4. DirectComposition DWM SwapChain Presentation.

---

## Future Work
- Add DirectX 12 Agility SDK integration for multi-GPU explicit adapter targeting.

## Known Issues
- None.

## References
- [crates/core_engine/src/rendering/mod.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/rendering/mod.rs)

## Related Documents
- [Rendering.md](../01_Architecture/Rendering.md)
