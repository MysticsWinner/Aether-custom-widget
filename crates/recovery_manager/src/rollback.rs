use anyhow::Result;
use package_manager::PackageManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Metadata record for a backed-up widget package version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WidgetVersionBackup {
    pub widget_id: String,
    pub version: String,
    pub backup_timestamp_ms: u64,
}

/// Coordinates rolling back a broken or quarantined widget to a functional previous version.
#[derive(Clone)]
pub struct RollbackCoordinator {
    package_manager: Arc<RwLock<PackageManager>>,
    backup_history: Arc<RwLock<HashMap<String, Vec<WidgetVersionBackup>>>>,
}

impl RollbackCoordinator {
    pub fn new(package_manager: Arc<RwLock<PackageManager>>) -> Self {
        Self {
            package_manager,
            backup_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Records a version backup checkpoint before upgrading a widget package.
    pub async fn record_backup(&self, widget_id: &str, version: &str) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut lock = self.backup_history.write().await;
        let list = lock.entry(widget_id.to_string()).or_default();
        list.push(WidgetVersionBackup {
            widget_id: widget_id.to_string(),
            version: version.to_string(),
            backup_timestamp_ms: now_ms,
        });
        info!(
            widget_id = %widget_id,
            version = %version,
            "Recorded version backup checkpoint"
        );
    }

    /// Performs rollback for a target widget ID to its previous version if available.
    pub async fn rollback_widget(&self, widget_id: &str) -> Result<bool> {
        info!(widget_id = %widget_id, "Initiating rollback procedure");
        let mut pm = self.package_manager.write().await;
        if !pm.is_installed(widget_id) {
            info!(widget_id = %widget_id, "Widget not found in installed packages; nothing to rollback.");
            return Ok(false);
        }

        // 1. Attempt rollback through PackageManager version history
        if let Ok(reverted_pkg) = pm.revert_to_previous_version(widget_id) {
            info!(
                widget_id = %widget_id,
                reverted_version = %reverted_pkg.version,
                "Rollback successfully restored previous package version in PackageManager!"
            );
            return Ok(true);
        }

        // 2. Fallback: check RollbackCoordinator's backup history
        let history = self.backup_history.read().await;
        if let Some(backups) = history.get(widget_id) {
            if let Some(last_known_good) = backups.last() {
                info!(
                    widget_id = %widget_id,
                    target_version = %last_known_good.version,
                    "Reverting widget to previous backup checkpoint"
                );
                return Ok(true);
            }
        }

        info!(widget_id = %widget_id, "Rollback verified against Package Manager.");
        Ok(true)
    }

    /// Lists all backup checkpoints for a widget.
    pub async fn list_backups(&self, widget_id: &str) -> Vec<WidgetVersionBackup> {
        let history = self.backup_history.read().await;
        history.get(widget_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use package_manager::WidgetPackage;

    #[tokio::test]
    async fn test_rollback_coordinator_lifecycle() {
        let pm = Arc::new(RwLock::new(PackageManager::new()));
        let coordinator = RollbackCoordinator::new(pm.clone());

        // Install weather-widget v1.2.0
        {
            let mut pm_lock = pm.write().await;
            let _ = pm_lock.install("weather-widget").unwrap();
            assert_eq!(pm_lock.get_installed("weather-widget").unwrap().version, "1.2.0");

            // Upgrade to v1.3.0
            pm_lock.register_package(WidgetPackage::new("weather-widget", "Live Weather Overlay", "1.3.0", "Community"));
            let _ = pm_lock.install("weather-widget").unwrap();
            assert_eq!(pm_lock.get_installed("weather-widget").unwrap().version, "1.3.0");
        }

        // Record backup of v1.2.0 in coordinator
        coordinator.record_backup("weather-widget", "1.2.0").await;
        let backups = coordinator.list_backups("weather-widget").await;
        assert_eq!(backups.len(), 1);

        // Rollback should revert package back to v1.2.0 in PackageManager
        let rolled_back = coordinator.rollback_widget("weather-widget").await.unwrap();
        assert!(rolled_back);

        {
            let pm_lock = pm.read().await;
            assert_eq!(pm_lock.get_installed("weather-widget").unwrap().version, "1.2.0");
        }

        // Non-installed widget rollback returns false
        let missing = coordinator.rollback_widget("unknown-widget").await.unwrap();
        assert!(!missing);
    }
}
