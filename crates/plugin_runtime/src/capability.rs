use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::warn;

/// Granular capability permission flags for sandboxed plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    /// Permission to read system hardware telemetry streams.
    ReadTelemetry,
    /// Permission to make restricted external HTTP requests.
    NetworkAccess,
    /// Permission to read/write files in scoped widget directory.
    FileSystemAccess,
    /// Permission to subscribe to system display/theme change events.
    SystemHooks,
    /// Permission to query custom GPU/NVML sensors.
    CustomHardware,
}

/// Manifest declaring requested and granted capabilities for a plugin package.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionManifest {
    pub plugin_id: String,
    pub requested_capabilities: HashSet<PluginCapability>,
    pub granted_capabilities: HashSet<PluginCapability>,
}

impl PermissionManifest {
    pub fn new(plugin_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            requested_capabilities: HashSet::new(),
            granted_capabilities: HashSet::new(),
        }
    }

    pub fn request_capability(&mut self, cap: PluginCapability) -> &mut Self {
        self.requested_capabilities.insert(cap);
        self
    }

    pub fn grant_capability(&mut self, cap: PluginCapability) -> &mut Self {
        self.granted_capabilities.insert(cap);
        self
    }
}

/// Security guard for validating permission access at runtime.
pub struct PermissionGuard;

impl PermissionGuard {
    /// Authorizes a requested capability against a plugin's permission manifest.
    pub fn authorize(manifest: &PermissionManifest, cap: PluginCapability) -> bool {
        if manifest.granted_capabilities.contains(&cap) {
            true
        } else {
            warn!(
                "Permission Denied: Plugin '{}' attempted unauthorized capability '{:?}'",
                manifest.plugin_id, cap
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_guard_authorization() {
        let mut manifest = PermissionManifest::new("plugin.weather.v1");
        manifest.request_capability(PluginCapability::NetworkAccess);
        manifest.grant_capability(PluginCapability::NetworkAccess);

        assert!(PermissionGuard::authorize(&manifest, PluginCapability::NetworkAccess));
        assert!(!PermissionGuard::authorize(&manifest, PluginCapability::FileSystemAccess));
    }
}
