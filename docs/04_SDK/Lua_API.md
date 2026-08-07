# Sandboxed Lua 5.4 Widget API Reference (`lua_runtime`)

**Purpose**: Documentation for Lua 5.4 scripting environment and helper functions.  
**Audience**: Lua Script Developers, Rainmeter Skin Authors.  
**Prerequisites**: [Widget_SDK.md](Widget_SDK.md).  
**Related Documents**: [Plugin_Runtime.md](../02_Core/Plugin_Runtime.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / API Reference  
**Owner**: Scripting Runtime Team  

---

## 1. Registered Global Lua Telemetry Functions

- `get_cpu_pct()` -> `number`
- `get_gpu_pct()` -> `number`
- `get_memory_mb()` -> `used_mb, total_mb`
- `get_net_rate()` -> `number`
- `get_open_apps_count()` -> `number`
- `get_browser_tabs_count()` -> `number`
- `get_master_volume_pct()` -> `number`
- `get_battery_charge_pct()` -> `number`
- `get_battery_remaining_secs()` -> `number`
- `get_gpu_count()` -> `number`
- `get_display_count()` -> `number`

---

## Future Work
- Add Rainmeter `.ini` skin auto-converter script in Lua.

## Known Issues
- None.

## References
- [crates/lua_runtime/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/lua_runtime/src/lib.rs)

## Related Documents
- [Widget_SDK.md](Widget_SDK.md)
