use std::sync::Arc;
use async_trait::async_trait;
use package_manager::PackageManager;
use tracing::info;
use crate::event_bus::EventBus;
use crate::subsystems::{Subsystem, SubsystemHealth};

/// Core Engine Subsystem wrapping the Phase 12 Marketplace Package Manager.
pub struct MarketplaceSubsystem {
    package_manager: PackageManager,
    event_bus: Option<Arc<EventBus>>,
}

impl MarketplaceSubsystem {
    pub fn new() -> Self {
        Self {
            package_manager: PackageManager::new(),
            event_bus: None,
        }
    }

    pub fn install_package(&mut self, package_name: &str) -> anyhow::Result<()> {
        self.package_manager.install(package_name)?;
        Ok(())
    }

    pub fn list_installed(&self) -> Vec<String> {
        self.package_manager
            .list()
            .iter()
            .map(|p| p.id.clone())
            .collect()
    }
}

impl Default for MarketplaceSubsystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subsystem for MarketplaceSubsystem {
    fn name(&self) -> &'static str {
        "marketplace_package_manager"
    }

    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing Phase 12 Marketplace Package Manager Subsystem...");
        self.event_bus = Some(bus);

        // Pre-install demo packages requested by user: install weather-widget, install spotify-widget, install taskbar-plus
        let _ = self.install_package("weather-widget");
        let _ = self.install_package("spotify-widget");
        let _ = self.install_package("taskbar-plus");

        info!(
            "Marketplace Package Manager initialized. Active installed packages: {:?}",
            self.list_installed()
        );

        Ok(())
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("MarketplaceSubsystem shut down cleanly.");
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
    async fn test_marketplace_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let mut subsystem = MarketplaceSubsystem::new();

        assert_eq!(subsystem.name(), "marketplace_package_manager");
        assert!(subsystem.initialize(bus).await.is_ok());

        let installed = subsystem.list_installed();
        assert!(installed.contains(&"weather-widget".to_string()));
        assert!(installed.contains(&"spotify-widget".to_string()));
        assert!(installed.contains(&"taskbar-plus".to_string()));

        assert!(subsystem.shutdown().await.is_ok());
    }
}
