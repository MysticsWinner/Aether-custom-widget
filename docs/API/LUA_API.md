# Sandboxed Lua 5.4 Widget API Reference (`lua_runtime`)

**Embedded Lua Scripting Bridge, Global Bindings, and Memory Limits**

---

## 1. Lua Environment & Sandbox

The `lua_runtime` crate hosts Lua 5.4 scripts inside an isolated memory context with restricted globals:
- `io`, `os.execute`, `package.loadlib` are **disabled** for security.
- Hard memory quota: max 16 MB per Lua state.

---

## 2. Aether Lua Global Table (`aether.*`)

| API Function | Description | Return Type |
|:---|:---|:---|
| `aether.get_cpu()` | Returns current CPU usage percentage | `number` (0.0 – 100.0) |
| `aether.get_ram()` | Returns `{ used_mb, total_mb, free_mb }` | `table` |
| `aether.get_gpu()` | Returns current GPU usage percentage | `number` (0.0 – 100.0) |
| `aether.draw_text(text, font, size, x, y, color)` | Emits a text draw command | `void` |
| `aether.draw_rect(x, y, w, h, color, radius)` | Emits a rounded rectangle draw command | `void` |
| `aether.log(msg)` | Writes to engine tracing stream | `void` |
