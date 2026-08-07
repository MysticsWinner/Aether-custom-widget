# Animation Engine Subsystem (`animation_engine`)

**Purpose**: Easing curves, spring physics, and timeline animation evaluation.  
**Audience**: UI Developers, Animation Designers.  
**Prerequisites**: [Widget_SDK.md](../04_SDK/Widget_SDK.md).  
**Related Documents**: [Widget_Runtime.md](Widget_Runtime.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: UI/UX Team  

---

## 1. Spring Physics Model (`SpringAnimation`)

Implements Hooke's Law with damping ratio ($\zeta$) and natural frequency ($\omega_0$):

$$F = -k(x - x_0) - c v$$

Provides fluid, natural UI motion for widget layout transitions.

---

## Future Work
- Add keyframe timeline editor evaluation primitives.

## Known Issues
- None.

## References
- [crates/animation_engine/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/animation_engine/src/lib.rs)

## Related Documents
- [Widget_SDK.md](../04_SDK/Widget_SDK.md)
