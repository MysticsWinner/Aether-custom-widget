use std::sync::Arc;
use async_trait::async_trait;
use tracing::info;
use crate::event_bus::EventBus;
use crate::profiler::{PerformanceProfileReport, SystemProfiler};
use crate::subsystems::{Subsystem, SubsystemHealth};

/// Core Engine Subsystem wrapping the Phase 11 Performance Profiler Engine.
pub struct ProfilerSubsystem {
    profiler: SystemProfiler,
    event_bus: Option<Arc<EventBus>>,
}

impl ProfilerSubsystem {
    pub fn new() -> Self {
        Self {
            profiler: SystemProfiler::new(),
            event_bus: None,
        }
    }

    pub fn sample_report(&self) -> PerformanceProfileReport {
        self.profiler.sample_profile()
    }
}

impl Default for ProfilerSubsystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subsystem for ProfilerSubsystem {
    fn name(&self) -> &'static str {
        "performance_profiler"
    }

    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing Phase 11 Performance Profiler Subsystem (13 Metrics Telemetry)...");
        self.event_bus = Some(bus);
        Ok(())
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        let report = self.profiler.sample_profile();
        if !report.nfr_compliance_passed {
            tracing::warn!("Performance Audit Warning: Non-functional requirement target limit breached.");
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("ProfilerSubsystem shut down cleanly.");
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
    async fn test_profiler_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let mut subsystem = ProfilerSubsystem::new();

        assert_eq!(subsystem.name(), "performance_profiler");
        assert!(subsystem.initialize(bus).await.is_ok());
        assert!(subsystem.tick().await.is_ok());

        let report = subsystem.sample_report();
        assert!(report.nfr_compliance_passed);

        assert!(subsystem.shutdown().await.is_ok());
    }
}
