# Security Architecture & Model (`capability_broker` & `plugin_runtime`)

**Purpose**: Master zero-trust security model, permission capability broker, and threat mitigations.  
**Audience**: Security Engineers, Core Developers.  
**Prerequisites**: [Architecture_Overview.md](../00_Project/Architecture_Overview.md).  
**Related Documents**: [Sandboxing.md](Sandboxing.md), [Threat_Model.md](Threat_Model.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Canonical Security Guide  
**Owner**: Security Architecture Lead  

---

## 1. Zero-Trust Security Model

Aether operates under a zero-trust model:
1. **Zero Direct Win32 API Access**: Widgets read exclusively from `SharedTelemetryCache`.
2. **AppContainer Sandboxing**: 3rd-party widget code runs out-of-process in restricted AppContainer SIDs.
3. **Revocable Capabilities**: All hardware, network, and disk accesses require explicit grant tokens issued by `CapabilityBroker`.
4. **BLAKE3 Binary Integrity**: `plugin_runtime` monitors plugin binary hashes to prevent runtime code injection.

---

## Future Work
- Add Windows Virtualization-Based Security (VBS) enclave isolation support.

## Known Issues
- None.

## References
- [crates/capability_broker/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/capability_broker/src/lib.rs)

## Related Documents
- [Sandboxing.md](Sandboxing.md)
- [Threat_Model.md](Threat_Model.md)
