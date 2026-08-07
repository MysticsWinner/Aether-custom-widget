# Memory & Allocation Architecture

**Purpose**: Explains memory allocation models, buffer pooling, and LRU resource caching in Aether.  
**Audience**: Performance Engineers, Systems Developers.  
**Prerequisites**: [Threading_Model.md](Threading_Model.md).  
**Related Documents**: [Performance.md](../08_Testing/Performance.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Core Architecture Team  

---

## 1. Zero-Allocation Metrics Read Model

Widget rendering loops query cached metrics directly from `SharedTelemetryCache` without heap allocations during active ticks.

---

## 2. LRU Resource Caching (`LruResourceCache`)

Font glyphs and bitmap assets are cached in `LruResourceCache` with automatic capacity-based eviction to enforce maximum physical RAM limits (< 25 MB footprint).

---

## Future Work
- Integrate custom bump allocator for transient draw command batches.

## Known Issues
- None.

## References
- [crates/widget_sdk/src/resource_cache.rs](file:///d:/Code/Aether-custom-widget/crates/widget_sdk/src/resource_cache.rs)

## Related Documents
- [Threading_Model.md](Threading_Model.md)
