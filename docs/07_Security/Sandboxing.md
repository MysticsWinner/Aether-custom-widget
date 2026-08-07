# AppContainer Sandboxing Architecture (`plugin_runtime`)

**Purpose**: Technical details of Windows AppContainer process isolation and fault recovery.  
**Audience**: Security Engineers, Platform Developers.  
**Prerequisites**: [Security_Model.md](Security_Model.md).  
**Related Documents**: [Permissions.md](Permissions.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Technical Specification  
**Owner**: Security & Runtime Team  

---

## 1. AppContainer Isolation Mechanism

Plugins execute in isolated worker processes spawned with low-privilege AppContainer SIDs (`CreateAppContainerProfile`). Access to user documents, system registry, and raw network sockets is denied by OS-level access control lists (ACLs).

---

## Future Work
- Add Win32 Job Object hard RAM limits per sandbox process.

## Known Issues
- None.

## References
- [crates/plugin_runtime/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/plugin_runtime/src/lib.rs)

## Related Documents
- [Security_Model.md](Security_Model.md)
