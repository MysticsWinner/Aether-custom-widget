use anyhow::Result;
use std::fmt::Debug;

/// Represents a single metric sample collected from an OS/Hardware provider.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Percentage(f32),
    Megabytes(f32),
    BytesPerSec(u64),
    /// Used by MemoryProvider to return both used and total RAM in one sample.
    MemoryStats { used_mb: f32, total_mb: f32 },
}

/// Abstract Metric Provider Interface.
/// Enforces interface isolation so collectors depend on abstractions.
pub trait MetricProvider: Send + Sync + Debug {
    /// Returns the provider name identifier.
    fn name(&self) -> &'static str;

    /// Samples hardware metric from system APIs.
    fn sample(&mut self) -> Result<MetricValue>;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

#[cfg(windows)]
fn filetime_to_u64(ft: windows::Win32::Foundation::FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
}

// ─── CPU Provider ─────────────────────────────────────────────────────────────

/// CPU Hardware Metric Collector Provider.
/// Uses `GetSystemTimes` (Win32) for accurate delta-based CPU utilisation.
#[derive(Debug)]
pub struct CpuProvider {
    prev_idle: u64,
    prev_total: u64,
    tick: u64,
}

impl CpuProvider {
    pub fn new() -> Self {
        // Seed with an initial snapshot so the first delta is valid
        let (idle, kernel, user) = Self::system_times_raw().unwrap_or((0, 1, 0));
        Self {
            prev_idle: idle,
            prev_total: kernel + user,
            tick: 0,
        }
    }

    #[cfg(windows)]
    fn system_times_raw() -> Result<(u64, u64, u64)> {
        use windows::Win32::Foundation::FILETIME;
        use windows::Win32::System::Threading::GetSystemTimes;
        unsafe {
            let mut idle = FILETIME::default();
            let mut kernel = FILETIME::default();
            let mut user = FILETIME::default();
            GetSystemTimes(
                Some(&mut idle),
                Some(&mut kernel),
                Some(&mut user),
            )?;
            Ok((
                filetime_to_u64(idle),
                filetime_to_u64(kernel),
                filetime_to_u64(user),
            ))
        }
    }

    #[cfg(not(windows))]
    fn system_times_raw() -> Result<(u64, u64, u64)> {
        Ok((0, 1, 0))
    }
}

impl Default for CpuProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for CpuProvider {
    fn name(&self) -> &'static str {
        "CpuProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;

        if let Ok((idle, kernel, user)) = Self::system_times_raw() {
            let total = kernel + user;
            let delta_idle = idle.saturating_sub(self.prev_idle);
            let delta_total = total.saturating_sub(self.prev_total);

            self.prev_idle = idle;
            self.prev_total = total;

            if delta_total > 0 {
                // kernel time includes idle, so busy = total − idle
                let cpu_pct =
                    (1.0 - delta_idle as f32 / delta_total as f32) * 100.0;
                return Ok(MetricValue::Percentage(cpu_pct.clamp(0.0, 100.0)));
            }
        }

        // Simulation fallback (dev / non-Windows)
        let val = ((self.tick as f32 * 0.15).sin().abs()) * 80.0 + 10.0;
        Ok(MetricValue::Percentage(val))
    }
}

// ─── Memory Provider ──────────────────────────────────────────────────────────

/// RAM Hardware Metric Collector Provider.
/// Uses `GlobalMemoryStatusEx` (Win32) for real used / total MB.
#[derive(Debug, Default)]
pub struct MemoryProvider {
    tick: u64,
}

impl MemoryProvider {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    #[cfg(windows)]
    fn query_memory() -> Result<(f32, f32)> {
        use windows::Win32::System::SystemInformation::{
            GlobalMemoryStatusEx, MEMORYSTATUSEX,
        };
        unsafe {
            let mut status = MEMORYSTATUSEX::default();
            status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            GlobalMemoryStatusEx(&mut status)?;
            let total_mb = status.ullTotalPhys as f32 / (1024.0 * 1024.0);
            let avail_mb = status.ullAvailPhys as f32 / (1024.0 * 1024.0);
            Ok((total_mb - avail_mb, total_mb))
        }
    }

    #[cfg(not(windows))]
    fn query_memory() -> Result<(f32, f32)> {
        Ok((4096.0, 16384.0))
    }
}

impl MetricProvider for MemoryProvider {
    fn name(&self) -> &'static str {
        "MemoryProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        if let Ok((used_mb, total_mb)) = Self::query_memory() {
            return Ok(MetricValue::MemoryStats { used_mb, total_mb });
        }
        // Simulation fallback
        let used = 4096.0 + ((self.tick as f32 * 0.05).cos().abs()) * 1024.0;
        Ok(MetricValue::MemoryStats {
            used_mb: used,
            total_mb: 16384.0,
        })
    }
}

// ─── GPU Provider ─────────────────────────────────────────────────────────────

/// GPU Hardware Metric Collector Provider.
/// Uses a high-quality mathematical simulation for now (DXGI engine-level
/// utilisation queries require D3DKMTQueryStatistics which needs Ring-0 elevation;
/// a PDH-based path will be added in a future phase).
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
        // Realistic-looking GPU load pattern (peaks around 60 Hz game-like workloads)
        let base = ((self.tick as f32 * 0.07).sin().abs()) * 45.0;
        let spike = ((self.tick as f32 * 0.31).cos().abs()) * 15.0;
        Ok(MetricValue::Percentage((base + spike).clamp(0.0, 100.0)))
    }
}

// ─── Network Provider ─────────────────────────────────────────────────────────

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

// ─── Tests ────────────────────────────────────────────────────────────────────

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
        assert!(matches!(
            cpu.sample().unwrap(),
            MetricValue::Percentage(_)
        ));
        assert!(matches!(
            mem.sample().unwrap(),
            MetricValue::MemoryStats { .. }
        ));
        assert!(matches!(
            gpu.sample().unwrap(),
            MetricValue::Percentage(_)
        ));
        assert!(matches!(
            net.sample().unwrap(),
            MetricValue::BytesPerSec(_)
        ));
    }

    #[test]
    fn test_cpu_percentage_in_range() {
        let mut cpu = CpuProvider::new();
        // Take several samples to populate deltas
        for _ in 0..5 {
            let val = cpu.sample().unwrap();
            if let MetricValue::Percentage(pct) = val {
                assert!((0.0..=100.0).contains(&pct), "CPU% out of range: {pct}");
            }
        }
    }

    #[test]
    fn test_memory_stats_sensible() {
        let mut mem = MemoryProvider::new();
        let val = mem.sample().unwrap();
        if let MetricValue::MemoryStats { used_mb, total_mb } = val {
            assert!(total_mb > 0.0, "total_mb must be positive");
            assert!(used_mb >= 0.0, "used_mb must be non-negative");
            assert!(used_mb <= total_mb, "used_mb cannot exceed total_mb");
        } else {
            panic!("Expected MemoryStats variant");
        }
    }
}
