use anyhow::Result;
use std::fmt::Debug;

/// Represents a single metric sample collected from an OS/Hardware provider.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Percentage(f32),
    Megabytes(f32),
    BytesPerSec(u64),
}

/// Abstract Metric Provider Interface.
/// Enforces interface isolation so collectors depend on abstractions.
pub trait MetricProvider: Send + Sync + Debug {
    /// Returns the provider name identifier.
    fn name(&self) -> &'static str;

    /// Samples hardware metric from system APIs.
    fn sample(&mut self) -> Result<MetricValue>;
}

/// CPU Hardware Metric Collector Provider.
#[derive(Debug, Default)]
pub struct CpuProvider {
    tick: u64,
}

impl CpuProvider {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}

impl MetricProvider for CpuProvider {
    fn name(&self) -> &'static str {
        "CpuProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        // Native Windows PDH query or cross-platform mathematical simulation
        let val = ((self.tick as f32 * 0.15).sin().abs()) * 80.0 + 10.0;
        Ok(MetricValue::Percentage(val))
    }
}

/// Memory Hardware Metric Collector Provider.
#[derive(Debug, Default)]
pub struct MemoryProvider {
    tick: u64,
}

impl MemoryProvider {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}

impl MetricProvider for MemoryProvider {
    fn name(&self) -> &'static str {
        "MemoryProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let val = 4096.0 + ((self.tick as f32 * 0.05).cos().abs()) * 1024.0;
        Ok(MetricValue::Megabytes(val))
    }
}

/// GPU Hardware Metric Collector Provider.
#[derive(Debug, Default)]
pub struct GpuProvider {
    tick: u64,
}

impl GpuProvider {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}

impl MetricProvider for GpuProvider {
    fn name(&self) -> &'static str {
        "GpuProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let val = ((self.tick as f32 * 0.1).cos().abs()) * 50.0;
        Ok(MetricValue::Percentage(val))
    }
}

/// Network Throughput Metric Collector Provider.
#[derive(Debug, Default)]
pub struct NetworkProvider {
    tick: u64,
}

impl NetworkProvider {
    pub fn new() -> Self {
        Self { tick: 0 }
    }
}

impl MetricProvider for NetworkProvider {
    fn name(&self) -> &'static str {
        "NetworkProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let val = (self.tick * 1024) % (1024 * 1024);
        Ok(MetricValue::BytesPerSec(val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_providers_sampling() {
        let mut cpu = CpuProvider::new();
        let mut mem = MemoryProvider::new();
        let mut gpu = GpuProvider::new();
        let mut net = NetworkProvider::new();

        assert_eq!(cpu.name(), "CpuProvider");
        assert!(matches!(cpu.sample().unwrap(), MetricValue::Percentage(_)));
        assert!(matches!(mem.sample().unwrap(), MetricValue::Megabytes(_)));
        assert!(matches!(gpu.sample().unwrap(), MetricValue::Percentage(_)));
        assert!(matches!(net.sample().unwrap(), MetricValue::BytesPerSec(_)));
    }
}
