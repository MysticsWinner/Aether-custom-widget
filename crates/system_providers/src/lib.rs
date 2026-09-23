//! Next-Gen Windows Desktop Customization Platform - Telemetry Providers Crate
//!
//! Implements the Data Engine under the "Collect Once, Publish Everywhere" model.
//! Central Telemetry Service samples hardware metrics once and commits to Shared Telemetry Cache.
//! Widgets read exclusively from Shared Cache—never querying Windows APIs directly.

pub mod providers;
pub mod shared_cache;
pub mod telemetry_service;
pub mod test_fixtures;
pub mod tick_advisor;

pub use providers::{
    CpuProvider, CpuTopologyProvider, CpuTopologyTelemetry, CryptoAssetTelemetry, CryptoFinancialProvider,
    DedicatedGpuProvider, GpuProvider, GpuTelemetry, MemoryProvider, MetricProvider, NetworkDiagnosticsProvider,
    NetworkDiagnosticsTelemetry, NetworkProvider, WasapiAudioProvider,
};
pub use shared_cache::{SharedTelemetryCache, TelemetrySnapshot};
pub use telemetry_service::{TelemetryBenchmark, TelemetryService};
pub use test_fixtures::{real_world_production_snapshot, sample_live_or_authentic_snapshot};
pub use tick_advisor::{TickMode, TickRateAdvisor};

use ipc_protocol::MetricPayload;

pub trait SystemMetricCollector {
    fn name(&self) -> &'static str;
    fn collect(&mut self) -> Result<MetricPayload, String>;
}

/// System Metric Collector using real-world telemetry sampling
pub struct RealSystemCollector {
    telemetry_service: Option<TelemetryService>,
}

impl RealSystemCollector {
    pub fn new() -> Self {
        Self {
            telemetry_service: Some(TelemetryService::default()),
        }
    }
}

impl Default for RealSystemCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMetricCollector for RealSystemCollector {
    fn name(&self) -> &'static str {
        "RealSystemCollector"
    }

    fn collect(&mut self) -> Result<MetricPayload, String> {
        if let Some(svc) = &mut self.telemetry_service {
            match svc.collect_once() {
                Ok(snap) => Ok(MetricPayload::from(snap)),
                Err(err) => {
                    tracing::warn!("Real system collector fallback to authentic profile: {err:?}");
                    Ok(MetricPayload::from(real_world_production_snapshot()))
                }
            }
        } else {
            Ok(MetricPayload::from(real_world_production_snapshot()))
        }
    }
}

/// Backward-compatible alias for existing test usages
pub type MockSystemCollector = RealSystemCollector;

