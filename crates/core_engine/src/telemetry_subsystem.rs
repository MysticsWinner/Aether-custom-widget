use std::sync::Arc;
use async_trait::async_trait;
use system_providers::{SharedTelemetryCache, TelemetryService};
use tracing::info;
use crate::event_bus::{CoreEvent, EventBus};
use crate::subsystems::{Subsystem, SubsystemHealth};

/// Core Engine Subsystem wrapping the Phase 7 Data Engine Telemetry Service.
pub struct TelemetrySubsystem {
    telemetry_service: TelemetryService,
    event_bus: Option<Arc<EventBus>>,
}

impl TelemetrySubsystem {
    /// Creates a new `TelemetrySubsystem` with a new `SharedTelemetryCache`.
    pub fn new() -> (Self, SharedTelemetryCache) {
        let cache = SharedTelemetryCache::new();
        let service = TelemetryService::new(cache.clone());
        (
            Self {
                telemetry_service: service,
                event_bus: None,
            },
            cache,
        )
    }

    /// Creates a new `TelemetrySubsystem` with an existing `SharedTelemetryCache`.
    pub fn with_cache(cache: SharedTelemetryCache) -> Self {
        let service = TelemetryService::new(cache);
        Self {
            telemetry_service: service,
            event_bus: None,
        }
    }
}

#[async_trait]
impl Subsystem for TelemetrySubsystem {
    fn name(&self) -> &'static str {
        "telemetry_data_engine"
    }

    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing Phase 7 Telemetry Subsystem (Collect Once, Publish Everywhere)...");
        self.event_bus = Some(bus);
        Ok(())
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        // Collect ONCE per sampling tick
        let snapshot = self.telemetry_service.collect_once()?;

        // Broadcast telemetry tick event to all interested subscribers (e.g. Layout, Render, Dashboard IPC)
        if let Some(bus) = &self.event_bus {
            let _ = bus.publish(CoreEvent::TelemetryTick {
                metric_id: "sys.cpu_usage".to_string(),
                value: snapshot.cpu_usage_pct as f64,
            });
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("TelemetrySubsystem shut down cleanly.");
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
    async fn test_telemetry_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let (mut subsystem, cache) = TelemetrySubsystem::new();

        assert_eq!(subsystem.name(), "telemetry_data_engine");
        assert!(subsystem.initialize(bus).await.is_ok());

        assert_eq!(cache.update_count(), 0);
        assert!(subsystem.tick().await.is_ok());
        assert_eq!(cache.update_count(), 1);

        assert!(subsystem.shutdown().await.is_ok());
    }
}
