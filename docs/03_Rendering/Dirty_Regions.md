# Retained-Mode Dirty Regions (`dirty_rect.rs`)

**Purpose**: Dirty region tracking algorithm, bounding box merging, and frame culling.  
**Audience**: Rendering Engineers, Performance Leads.  
**Prerequisites**: [Rendering.md](../01_Architecture/Rendering.md).  
**Related Documents**: [Direct2D.md](Direct2D.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Graphics Team  

---

## 1. Merging Algorithm (`DirtyRegionTracker`)

`DirtyRegionTracker` merges overlapping rectangle regions into minimal bounding boxes, allowing the renderer to redraw only dirty screen pixels while skipping 90%+ of zero-delta areas.

---

## Future Work
- Add quad-tree spatial partitioning for multi-widget dirty region queries.

## Known Issues
- None.

## References
- [crates/core_engine/src/rendering/dirty_rect.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/rendering/dirty_rect.rs)

## Related Documents
- [Rendering.md](../01_Architecture/Rendering.md)
