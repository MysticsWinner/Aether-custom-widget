//! Next-Gen Windows Desktop Customization Platform - Telemetry Providers Crate
//!
//! Implements the Data Engine under the "Collect Once, Publish Everywhere" model.
//! Central Telemetry Service samples hardware metrics once and commits to Shared Telemetry Cache.
//! Widgets read exclusively from Shared Cache—never querying Windows APIs directly.

pub mod providers;
pub mod shared_cache;
pub mod telemetry_service;

pub use providers::{CpuProvider, GpuProvider, MemoryProvider, MetricProvider, NetworkProvider};
pub use shared_cache::{SharedTelemetryCache, TelemetrySnapshot};
pub use telemetry_service::{TelemetryBenchmark, TelemetryService};

use ipc_protocol::MetricPayload;

pub trait SystemMetricCollector {
    fn name(&self) -> &'static str;
    fn collect(&mut self) -> Result<MetricPayload, String>;
}

/// Simulated / Mock Hardware Collector for cross-compilation & test verification
pub struct MockSystemCollector {
    tick: u64,
}

impl MockSystemCollector {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}

impl Default for MockSystemCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMetricCollector for MockSystemCollector {
    fn name(&self) -> &'static str {
        "MockHardwareCollector"
    }

    fn collect(&mut self) -> Result<MetricPayload, String> {
        self.tick += 1;
        let mock_cpu = ((self.tick as f32 * 0.1).sin().abs()) * 100.0;

        Ok(MetricPayload {
            timestamp_ms: self.tick * 1000,
            cpu_usage_pct: mock_cpu,
            memory_used_mb: 8192.0,
            memory_total_mb: 16384.0,
            gpu_usage_pct: 12.5,
            net_recv_bytes_per_sec: 1024 * 50,
            net_sent_bytes_per_sec: 1024 * 12,
        })
    }
}
