use std::sync::Arc;
use async_trait::async_trait;
use plugin_runtime::{ApiVersion, PermissionManifest, PluginHealth, PluginSupervisor};
use tracing::info;
use crate::event_bus::EventBus;
use crate::subsystems::{Subsystem, SubsystemHealth};

/// Core Engine Subsystem wrapping the Phase 10 Plugin Sandbox Supervisor.
pub struct PluginSandboxSubsystem {
    supervisor: PluginSupervisor,
    event_bus: Option<Arc<EventBus>>,
}

impl PluginSandboxSubsystem {
    pub fn new() -> Self {
        Self {
            supervisor: PluginSupervisor::new(),
            event_bus: None,
        }
    }

    pub fn launch_plugin(
        &mut self,
        plugin_id: &str,
        required_api: ApiVersion,
        manifest: PermissionManifest,
    ) -> anyhow::Result<u32> {
        self.supervisor.launch_plugin(plugin_id, required_api, manifest)
    }

    pub fn simulate_crash(&mut self, plugin_id: &str, exit_code: i32) {
        self.supervisor.handle_plugin_crash(plugin_id, exit_code);
    }

    pub fn plugin_health(&self, plugin_id: &str) -> Option<PluginHealth> {
        self.supervisor.plugin_health(plugin_id)
    }
}

impl Default for PluginSandboxSubsystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subsystem for PluginSandboxSubsystem {
    fn name(&self) -> &'static str {
        "plugin_sandbox"
    }

    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing Phase 10 Plugin Sandbox Subsystem (AppContainer isolation & crash fault tolerance)...");
        self.event_bus = Some(bus);
        Ok(())
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("PluginSandboxSubsystem shut down cleanly.");
        Ok(())
    }

    fn health(&self) -> SubsystemHealth {
        SubsystemHealth::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let mut subsystem = PluginSandboxSubsystem::new();

        assert_eq!(subsystem.name(), "plugin_sandbox");
        assert!(subsystem.initialize(bus).await.is_ok());

        let manifest = PermissionManifest::new("sys.widget.clock");
        let pid = subsystem
            .launch_plugin("sys.widget.clock", ApiVersion::new(1, 0, 0), manifest)
            .unwrap();

        assert!(pid > 0);
        assert_eq!(subsystem.plugin_health("sys.widget.clock"), Some(PluginHealth::Running));

        subsystem.simulate_crash("sys.widget.clock", -1);
        assert_eq!(subsystem.plugin_health("sys.widget.clock"), Some(PluginHealth::Running));

        assert!(subsystem.shutdown().await.is_ok());
    }
}
