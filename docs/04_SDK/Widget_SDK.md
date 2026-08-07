# Standardized Widget SDK Guide (`widget_sdk`)

**Purpose**: Master guide for the 6-pillar widget development API surface.  
**Audience**: Widget Developers, Plugin Authors.  
**Prerequisites**: [Widget_Runtime.md](../02_Core/Widget_Runtime.md).  
**Related Documents**: [Rust_SDK.md](Rust_SDK.md), [Lua_API.md](Lua_API.md), [CSharp_SDK.md](CSharp_SDK.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Canonical SDK Guide  
**Owner**: SDK & Developer Relations Team  

---

## 1. The 6 SDK Pillars

1. **Lifecycle (`WidgetLifecycle`)**: Standard load, mount, update, unmount, unload hooks.
2. **Rendering (`RenderCanvas`)**: High-level primitive batching (`draw_rect`, `draw_text`).
3. **Settings (`SettingsStore`)**: Key-value settings persistence.
4. **Events (`EventSubscriber`)**: Async topic-based event subscriptions.
5. **Animation (`SpringAnimation`)**: Easing curves & spring physics.
6. **Resources (`ResourceManager`)**: Asset loading & memory management.

---

## Future Work
- Add high-level UI component framework (Buttons, Sliders, Switches).

## Known Issues
- None.

## References
- [crates/widget_sdk/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/widget_sdk/src/lib.rs)

## Related Documents
- [Rust_SDK.md](Rust_SDK.md)
- [Lua_API.md](Lua_API.md)
