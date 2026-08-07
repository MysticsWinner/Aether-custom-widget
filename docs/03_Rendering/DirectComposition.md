# DirectComposition Rendering Subsystem

**Purpose**: DirectComposition DWM surface composition target integration for Windows 11.  
**Audience**: Rendering Engineers, Windows Platform Developers.  
**Prerequisites**: [Rendering.md](../01_Architecture/Rendering.md).  
**Related Documents**: [Direct2D.md](Direct2D.md), [WorkerW.md](WorkerW.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Graphics Team  

---

## 1. DirectComposition Pipeline

DirectComposition composites visual trees directly into the Windows Desktop Window Manager (DWM) composition swap chain, achieving 144Hz+ tear-free hardware acceleration.

---

## Future Work
- Add DirectComposition visual transform animations.

## Known Issues
- None.

## References
- [crates/widget_sdk/src/rendering.rs](file:///d:/Code/Aether-custom-widget/crates/widget_sdk/src/rendering.rs)

## Related Documents
- [Direct2D.md](Direct2D.md)
