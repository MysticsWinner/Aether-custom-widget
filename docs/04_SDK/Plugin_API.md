# Low-Level Plugin C API & ABI Specification

**Purpose**: ABI specifications for native C/C++ FFI plugin extensions.  
**Audience**: C/C++ Developers, Low-Level Integrators.  
**Prerequisites**: [Widget_SDK.md](Widget_SDK.md).  
**Related Documents**: [Plugin_Runtime.md](../02_Core/Plugin_Runtime.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / ABI Specification  
**Owner**: Low-Level Systems Team  

---

## 1. C FFI Export Signatures

Native C/C++ plugins export `aether_plugin_init`, `aether_plugin_tick`, and `aether_plugin_shutdown` entry points loaded via `libloading` dynamic library calls.

---

## Future Work
- Add `cbindgen` auto-generated header file `aether_plugin.h`.

## Known Issues
- None.

## References
- [crates/plugin_runtime/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/plugin_runtime/src/lib.rs)

## Related Documents
- [Widget_SDK.md](Widget_SDK.md)
