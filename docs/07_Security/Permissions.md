# Permissions & Capability Broker Specification (`capability_broker`)

**Purpose**: Capability token structure, grant store persistence, and `WidgetFirewall` rules.  
**Audience**: Security Engineers, Widget Authors.  
**Prerequisites**: [Security_Model.md](Security_Model.md).  
**Related Documents**: [Sandboxing.md](Sandboxing.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Security Specification  
**Owner**: Security Team  

---

## 1. Capability Grant Model

`CapabilityBroker` manages fine-grained capability tokens (`CapabilityToken`):
- `ReadSystemTelemetry`
- `NetworkAccess { domain_whitelist }`
- `StorageAccess { isolated_dir_only: bool }`

Tokens are signed, revocable at runtime, and persisted in `GrantStore`.

---

## Future Work
- Add user-facing interactive permission prompt modal in WinUI 3 dashboard.

## Known Issues
- None.

## References
- [crates/capability_broker/src/broker.rs](file:///d:/Code/Aether-custom-widget/crates/capability_broker/src/broker.rs)

## Related Documents
- [Security_Model.md](Security_Model.md)
