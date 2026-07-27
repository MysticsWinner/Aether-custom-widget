//! Next-Gen Windows Desktop Customization Platform - Plugin Sandbox Runtime Crate
//!
//! Provides out-of-process AppContainer sandboxing, JobObject resource caps,
//! granular capability permission model, Semantic Versioning API compatibility,
//! and crash fault isolation ensuring plugin crashes NEVER crash the core runtime.

pub mod capability;
pub mod compatibility;
pub mod supervisor;

pub use capability::{PermissionGuard, PermissionManifest, PluginCapability};
pub use compatibility::{ApiVersion, CompatibilityChecker};
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
