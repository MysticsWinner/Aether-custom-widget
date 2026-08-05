# Aether — Lua Scripting API (`lua_runtime`)

**Lua 5.4 Scripting Integration and Host Function Registrations**

---

## 1. Lua Runtime Integration Architecture

The `lua_runtime` crate integrates `mlua 0.9` for executing lightweight, interpreted widget scripts without compiling native Rust binaries:

```rust
pub struct LuaEngine {
    lua: mlua::Lua,
}
```

---

## 2. Host Function Registrations

Currently registered host function in `lua_runtime::LuaEngine`:

- **`log_info(message: string)`**: Logs message string to host engine's `tracing` output.

### Planned Host API Expansion Example (Lua)
```lua
-- widget.lua
function on_update(ctx)
    local cpu_pct = Aether.telemetry.get_cpu_pct()
    Aether.canvas.draw_rect(0, 0, 200, 100, "#1E1E2E", 8)
    Aether.canvas.draw_text("CPU: " .. string.format("%.1f", cpu_pct) .. "%", 10, 10, 14, "#FFFFFF")
end
```
