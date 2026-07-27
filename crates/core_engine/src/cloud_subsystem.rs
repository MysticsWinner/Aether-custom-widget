use std::sync::Arc;
use async_trait::async_trait;
use cloud_sync::{
    AccountEntity, CloudSyncManager, DeviceEntity, LayoutEntity, PluginEntity, SettingsEntity,
    SyncEntity, ThemeEntity,
};
use tracing::info;
use crate::event_bus::EventBus;
use crate::subsystems::{Subsystem, SubsystemHealth};

/// Core Engine Subsystem wrapping the Phase 13 Cloud Sync Engine.
pub struct CloudSyncSubsystem {
    manager: CloudSyncManager,
    event_bus: Option<Arc<EventBus>>,
}

impl CloudSyncSubsystem {
    pub fn new() -> Self {
        Self {
            manager: CloudSyncManager::new("host_daemon_primary"),
            event_bus: None,
        }
    }

    pub fn sync_entity(&mut self, entity_id: &str, entity: SyncEntity) {
        self.manager.sync_entity(entity_id, entity);
    }
}

impl Default for CloudSyncSubsystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subsystem for CloudSyncSubsystem {
    fn name(&self) -> &'static str {
        "cloud_sync_engine"
    }

    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing Phase 13 Cloud Sync Engine Subsystem (CRDT Conflict Resolution & Offline Mode)...");
        self.event_bus = Some(bus);

        // Pre-sync samples across all 6 entities
        let layout = SyncEntity::Layout(LayoutEntity {
            layout_id: "layout.primary".into(),
            display_id: "DISPLAY_1".into(),
            bounds_x: 0.0,
            bounds_y: 0.0,
            width: 1920.0,
            height: 1080.0,
        });
        let theme = SyncEntity::Theme(ThemeEntity {
            theme_id: "theme.default.dark".into(),
            color_tokens: Default::default(),
            font_family: "Segoe UI".into(),
        });
        let settings = SyncEntity::Settings(SettingsEntity {
            setting_id: "setting.telemetry".into(),
            key: "telemetry_enabled".into(),
            value_json: "true".into(),
        });
        let plugin = SyncEntity::Plugin(PluginEntity {
            plugin_id: "weather-widget".into(),
            pinned_version: "1.0.0".into(),
            enabled: true,
        });
        let device = SyncEntity::Device(DeviceEntity {
            device_id: "workstation_pc".into(),
            hostname: "DESKTOP-WIN11".into(),
            monitor_count: 2,
            os_build: "22631".into(),
        });
        let account = SyncEntity::Account(AccountEntity {
            user_id: "user_77812".into(),
            email: "user@example.com".into(),
            auth_token_encrypted: "aes256gcm_encrypted_blob".into(),
        });

        self.sync_entity("layout.primary", layout);
        self.sync_entity("theme.default.dark", theme);
        self.sync_entity("setting.telemetry", settings);
        self.sync_entity("plugin.weather", plugin);
        self.sync_entity("device.pc", device);
        self.sync_entity("account.user", account);

        info!("Cloud Sync Engine initialized. 6 Core Entities synced with local cache & CRDT solver.");
        Ok(())
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("CloudSyncSubsystem shut down cleanly.");
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
    async fn test_cloud_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let mut subsystem = CloudSyncSubsystem::new();

        assert_eq!(subsystem.name(), "cloud_sync_engine");
        assert!(subsystem.initialize(bus).await.is_ok());
        assert!(subsystem.shutdown().await.is_ok());
    }
}
