use std::sync::Arc;
use async_trait::async_trait;
use theme_engine::{DynamicThemeStore, ThemeResolver, ThemeSchema};
use tracing::info;
use crate::event_bus::{CoreEvent, EventBus};
use crate::subsystems::{Subsystem, SubsystemHealth};

/// Core Engine Subsystem wrapping the Phase 9 Theme Engine (Dynamic theme.json & Hot Reloading).
pub struct ThemeEngineSubsystem {
    store: DynamicThemeStore,
    event_bus: Option<Arc<EventBus>>,
}

impl ThemeEngineSubsystem {
    pub fn new() -> (Self, DynamicThemeStore) {
        let store = DynamicThemeStore::default();
        (
            Self {
                store: store.clone(),
                event_bus: None,
            },
            store,
        )
    }

    pub fn with_store(store: DynamicThemeStore) -> Self {
        Self {
            store,
            event_bus: None,
        }
    }

    /// Triggers a hot swap of theme schema without restarting host daemon.
    pub fn hot_reload(&self, new_schema: ThemeSchema) {
        let name = new_schema.metadata.name.clone();
        self.store.hot_swap_schema(new_schema);

        if let Some(bus) = &self.event_bus {
            let _ = bus.publish(CoreEvent::ThemeChanged {
                theme_name: name,
            });
        }
    }
}

#[async_trait]
impl Subsystem for ThemeEngineSubsystem {
    fn name(&self) -> &'static str {
        "theme_engine"
    }

    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing Phase 9 Theme Engine Subsystem (theme.json & Hot Reloadable tokens)...");
        self.event_bus = Some(bus);
        Ok(())
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("ThemeEngineSubsystem shut down cleanly.");
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
    async fn test_theme_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let (mut subsystem, store) = ThemeEngineSubsystem::new();

        assert_eq!(subsystem.name(), "theme_engine");
        assert!(subsystem.initialize(bus).await.is_ok());

        assert_eq!(store.resolve_color("theme.accent"), "#0078D7");

        let mut new_schema = ThemeSchema::default();
        new_schema.colors.insert("theme.accent".into(), "#00FF00".into());
        subsystem.hot_reload(new_schema);

        assert_eq!(store.resolve_color("theme.accent"), "#00FF00");

        assert!(subsystem.shutdown().await.is_ok());
    }
}
