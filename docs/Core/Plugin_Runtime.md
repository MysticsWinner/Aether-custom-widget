# Plugin Runtime Subsystem (`plugin_runtime`)

**Purpose**: AppContainer process supervisor, WebAssembly linear memory sandbox, API version compatibility checker, and memory guard.  
**Audience**: Security Engineers, Plugin Developers.  
**Prerequisites**: [SECURITY_ARCHITECTURE.md](../Security/SECURITY_ARCHITECTURE.md).  
**Related Documents**: [SANDBOX.md](../Security/SANDBOX.md).  
**Last Updated**: 2026-09-01  
**Status**: Active / Core Subsystem  
**Owner**: Security & Runtime Team  

---

## 1. Multi-Target Sandboxing Architecture

The `plugin_runtime` provides two complementary sandboxing execution models:

1. **AppContainer Process Supervisor**:
   - Executes native `.dll` plugin binaries inside low-integrity Windows AppContainers.
   - Enforces JobObject quotas (2% CPU rate limit, 50 MB RAM cap).
   - Auto-respawns faulted worker processes in `< 5ms` without affecting the host engine daemon.

2. **WebAssembly (`wasm`) Sandbox Engine**:
   - Executes compiled `.wasm` bytecode inside memory-isolated linear memory sandboxes (64 KB pages).
   - Enables multi-language plugin authoring (**Rust**, **C/C++**, **Zig**, **Go**, **AssemblyScript**).
   - Fast host ABI dispatch (`aether_*`) with zero host memory corruption risk.

---

## Known Issues
- None.

## References
- [crates/plugin_runtime/src/lib.rs](../../crates/plugin_runtime/src/lib.rs)
- [crates/plugin_runtime/src/wasm.rs](../../crates/plugin_runtime/src/wasm.rs)
- [crates/plugin_runtime/src/supervisor.rs](../../crates/plugin_runtime/src/supervisor.rs)

## Related Documents
- [SANDBOX.md](../Security/SANDBOX.md)
