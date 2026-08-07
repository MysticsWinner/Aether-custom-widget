# Plugin Runtime Subsystem (`plugin_runtime`)

**Purpose**: AppContainer process supervisor, API version compatibility checker, and memory guard.  
**Audience**: Security Engineers, Plugin Developers.  
**Prerequisites**: [Security_Model.md](../07_Security/Security_Model.md).  
**Related Documents**: [Sandboxing.md](../07_Security/Sandboxing.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: Security & Runtime Team  

---

## 1. AppContainer Process Supervisor

`plugin_runtime` supervises sandboxed widget worker processes. If a plugin crashes or triggers an unhandled fault, the supervisor catches the process termination signal and auto-respawns the worker in < 5ms without affecting the host daemon.

---

## Future Work
- Integrate WASM runtime (`wasmtime`) as an alternative in-process sandboxing target.

## Known Issues
- None.

## References
- [crates/plugin_runtime/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/plugin_runtime/src/lib.rs)

## Related Documents
- [Sandboxing.md](../07_Security/Sandboxing.md)
