# Permissions & Capability Broker Specification

**Capability Tokens, Manifest Declarations, and Widget Firewall Rules**

---

## 1. Capability Permission Model

Widgets declare required system capabilities inside their `widget.toml` manifest:

```toml
[permissions]
network = false
filesystem = false
gpu_acceleration = true
telemetry_read = true
```

---

## 2. Capability Token Structure (`capability_broker`)

The `CapabilityBroker` manages fine-grained, cryptographically signed capability tokens (`CapabilityToken`):

- `ReadSystemTelemetry` — Grants zero-copy access to `SharedTelemetryCache`.
- `NetworkAccess { domain_whitelist }` — Grants access to explicit whitelist endpoints for weather/RSS widgets.
- `StorageAccess { isolated_dir_only: bool }` — Restricts file I/O strictly to `%LOCALAPPDATA%\Aether\Plugins\<plugin_id>\`.
- `GpuAcceleration` — Grants Direct2D / DirectComposition hardware rendering surfaces.

Tokens are signed, revocable at runtime, and persisted in `GrantStore`.

---

## 3. Widget Firewall Runtime Guard

The `WidgetFirewall` inside `capability_broker` checks capability permissions dynamically:

```rust
pub struct PermissionManifest {
    pub network_access: bool,
    pub filesystem_access: bool,
    pub gpu_access: bool,
}

impl PermissionManifest {
    pub fn can_access_network(&self) -> bool { self.network_access }
    pub fn can_access_filesystem(&self) -> bool { self.filesystem_access }
    pub fn can_use_gpu(&self) -> bool { self.gpu_access }
}
```

By default, all undeclared capabilities evaluate to `false` (strict deny-by-default posture).
