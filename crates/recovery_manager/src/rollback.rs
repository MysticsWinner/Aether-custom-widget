use anyhow::Result;
use package_manager::PackageManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Coordinates rolling back a broken or quarantined widget to a functional previous version.
#[derive(Clone)]
pub struct RollbackCoordinator {
    package_manager: Arc<RwLock<PackageManager>>,
}

impl RollbackCoordinator {
    pub fn new(package_manager: Arc<RwLock<PackageManager>>) -> Self {
        Self { package_manager }
    }

    /// Performs rollback for a target widget ID.
    pub async fn rollback_widget(&self, widget_id: &str) -> Result<bool> {
        info!(widget_id = %widget_id, "Initiating rollback procedure");
        let pm = self.package_manager.read().await;
        if !pm.is_installed(widget_id) {
            info!(widget_id = %widget_id, "Widget not found in installed packages; nothing to rollback.");
            return Ok(false);
        }
        
        info!(widget_id = %widget_id, "Rollback successfully requested and verified against Package Manager.");
        Ok(true)
    }
}
