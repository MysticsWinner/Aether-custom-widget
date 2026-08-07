# Layout Engine Subsystem (`layout_engine`)

**Purpose**: Flexbox layout computation and widget position storage.  
**Audience**: Engine Developers, UI Layout Engineers.  
**Prerequisites**: [System_Architecture.md](../01_Architecture/System_Architecture.md).  
**Related Documents**: [Widget_Runtime.md](Widget_Runtime.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: Layout & UI Team  

---

## 1. Widget Position Storage (`WidgetPositionStore`)

Manages persistence of widget drag-and-drop $(x, y)$ coordinates and lock flags (`is_locked`). Saves configuration changes atomically to `settings.json`.

---

## Future Work
- Add automatic magnetic snap-to-grid layout alignment.

## Known Issues
- None.

## References
- [crates/layout_engine/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/layout_engine/src/lib.rs)

## Related Documents
- [Widget_Runtime.md](Widget_Runtime.md)
