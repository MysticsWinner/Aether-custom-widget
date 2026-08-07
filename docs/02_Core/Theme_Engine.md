# Theme Engine Subsystem (`theme_engine`)

**Purpose**: JSON theme schema, file-watcher hot-swapper, and design token resolver.  
**Audience**: UI/UX Designers, Widget Developers.  
**Prerequisites**: [Widget_SDK.md](../04_SDK/Widget_SDK.md).  
**Related Documents**: [Engine.md](Engine.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: Design Systems Team  

---

## 1. Theme Specification & Token Resolution

`theme_engine` parses JSON theme manifests defining color design tokens (e.g. `bg.primary`, `accent.cyan`, `glass.opacity`). `ThemeResolver` resolves token references and queries Windows system accent colors via Win32 `DwmGetColorizationColor`.

---

## Future Work
- Add dynamic theme extraction from desktop wallpaper image dominant colors.

## Known Issues
- None.

## References
- [crates/theme_engine/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/theme_engine/src/lib.rs)

## Related Documents
- [Engine.md](Engine.md)
