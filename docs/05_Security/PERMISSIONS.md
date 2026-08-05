# Aether — Capability Permission Model

**Widget Permissions, Manifest Declarations, and Access Controls**

---

## 1. Permission Manifest Schema (`plugin_runtime::capability`)

Widgets declare required system capabilities inside their `widget.toml` manifest via `PermissionManifest`:

```toml
[permissions]
network = false
filesystem = false
gpu_acceleration = true
telemetry_read = true
```

---

## 2. Permission Evaluation Rules

The `PermissionGuard` inside `plugin_runtime` checks access permissions prior to executing API requests:

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

By default, all capabilities evaluate to `false` (deny-by-default security posture).
