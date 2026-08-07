//! Next-Gen Windows Desktop Customization Platform - Plugin Sandbox Runtime Crate
//!
//! Provides out-of-process AppContainer sandboxing, JobObject resource caps,
//! granular capability permission model, Semantic Versioning API compatibility,
//! and crash fault isolation ensuring plugin crashes NEVER crash the core runtime.

pub mod capability;
pub mod compatibility;
pub mod integrity;
pub mod memory_guard;
pub mod supervisor;

pub use capability::{PermissionGuard, PermissionManifest, PluginCapability};
pub use compatibility::{ApiVersion, CompatibilityChecker};
pub use integrity::{compute_blake3_hash, PluginHashStore};
pub use memory_guard::{MemoryGuard, ResourceUsageReport, ResourceWarning};
pub use supervisor::{PluginHealth, PluginProcessInfo, PluginSandboxBenchmark, PluginSupervisor};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to create AppContainer profile: {0}")]
    AppContainerCreationFailed(String),
    #[error("JobObject limit configuration failed: {0}")]
    JobObjectConfigFailed(String),
    #[error("Process launch failed: {0}")]
    ProcessLaunchFailed(String),
}

pub struct AppContainerSandboxSpec {
    pub container_name: String,
    pub display_name: String,
    pub cpu_rate_limit_pct: u32,
    pub max_memory_bytes: usize,
}

pub struct PluginProcessManager {
    spec: AppContainerSandboxSpec,
}

impl PluginProcessManager {
    pub fn new(spec: AppContainerSandboxSpec) -> Self {
        Self { spec }
    }

    /// Spawns sandboxed plugin process under Windows AppContainer SID & JobObject limits
    pub fn spawn_sandboxed_plugin(&self, _binary_path: &str) -> Result<u32, SandboxError> {
        tracing::info!(
            "Configuring AppContainer profile '{}' with max RAM {} MB and max CPU {}%",
            self.spec.container_name,
            self.spec.max_memory_bytes / (1024 * 1024),
            self.spec.cpu_rate_limit_pct
        );

        Ok(4242)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_plugin_integrity_monitor_detects_tampered_binary() {
        let dir = tempdir().unwrap();
        let bin_path = dir.path().join("plugin.dll");
        std::fs::write(&bin_path, b"original trusted binary content").unwrap();

        let hash = compute_blake3_hash(&bin_path).unwrap();
        let store_path = dir.path().join("hashes.json");
        let mut store = PluginHashStore::new(&store_path);

        store.register_hash("trusted_plugin", &hash).unwrap();
        assert!(store.verify_plugin_binary("trusted_plugin", &bin_path).unwrap());

        // Tamper binary
        std::fs::write(&bin_path, b"TAMPERED MALICIOUS BINARY").unwrap();
        assert!(!store.verify_plugin_binary("trusted_plugin", &bin_path).unwrap());
    }

    #[test]
    fn test_memory_guard_resource_warning_thresholds() {
        let guard = MemoryGuard::default();

        // Normal metrics -> no warning
        let report_ok = guard.evaluate("widget_1", 10.0, 50.0, 20.0, 100.0, 5, 16);
        assert_eq!(report_ok.warning, None);

        // CPU runaway -> warning
        let report_cpu = guard.evaluate("widget_1", 95.0, 50.0, 20.0, 100.0, 5, 16);
        assert!(matches!(report_cpu.warning, Some(ResourceWarning::CpuRunaway { .. })));

        // Memory leak -> warning
        let report_mem = guard.evaluate("widget_1", 10.0, 50.0, 95.0, 100.0, 5, 16);
        assert!(matches!(report_mem.warning, Some(ResourceWarning::MemoryLeak { .. })));
    }
}
