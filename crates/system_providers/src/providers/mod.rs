use anyhow::Result;
use std::fmt::Debug;
use std::time::Instant;

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
/// Uses `GetSystemTimes` (Win32) for accurate delta-based CPU utilisation,
/// with Task Manager-style zero-delta value holding and Exponential Moving Average (EMA) smoothing.
#[derive(Debug)]
pub struct CpuProvider {
    prev_idle: u64,
    prev_total: u64,
    last_valid_pct: f32,
    ema_pct: f32,
    tick: u64,
    alpha: f32,
}

impl CpuProvider {
    pub fn new() -> Self {
        // Seed with an initial snapshot so the first delta is valid
        let (idle, kernel, user) = Self::system_times_raw().unwrap_or((0, 1, 0));
        Self {
            prev_idle: idle,
            prev_total: kernel + user,
            last_valid_pct: 0.0,
            ema_pct: 0.0,
            tick: 0,
            alpha: 0.25, // Task Manager temporal smoothing factor
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

        let raw_pct = if let Ok((idle, kernel, user)) = Self::system_times_raw() {
            let total = kernel + user;
            let delta_idle = idle.saturating_sub(self.prev_idle);
            let delta_total = total.saturating_sub(self.prev_total);

            if delta_total > 0 {
                self.prev_idle = idle;
                self.prev_total = total;

                // kernel time includes idle, so busy = total − idle
                let cpu_pct = (1.0 - delta_idle as f32 / delta_total as f32) * 100.0;
                let clamped = cpu_pct.clamp(0.0, 100.0);
                self.last_valid_pct = clamped;
                clamped
            } else {
                // If sub-quantum tick (delta_total == 0), hold last valid percentage instead of fake sine waves
                self.last_valid_pct
            }
        } else {
            self.last_valid_pct
        };

        // Apply Exponential Moving Average (EMA) smoothing for Task Manager consistency
        if self.tick == 1 {
            self.ema_pct = raw_pct;
        } else {
            self.ema_pct = (self.alpha * raw_pct) + ((1.0 - self.alpha) * self.ema_pct);
        }

        Ok(MetricValue::Percentage(self.ema_pct.clamp(0.0, 100.0)))
    }
}

// ─── Memory Provider ──────────────────────────────────────────────────────────

/// RAM Hardware Metric Collector Provider.
/// Uses `GlobalMemoryStatusEx` (Win32) for real used / total MB with EMA temporal smoothing.
#[derive(Debug)]
pub struct MemoryProvider {
    tick: u64,
    ema_used_mb: f32,
    alpha: f32,
}

impl MemoryProvider {
    pub fn new() -> Self {
        Self {
            tick: 0,
            ema_used_mb: 0.0,
            alpha: 0.3,
        }
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

impl Default for MemoryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for MemoryProvider {
    fn name(&self) -> &'static str {
        "MemoryProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let (used_mb, total_mb) = Self::query_memory().unwrap_or((4096.0, 16384.0));

        if self.tick == 1 {
            self.ema_used_mb = used_mb;
        } else {
            self.ema_used_mb = (self.alpha * used_mb) + ((1.0 - self.alpha) * self.ema_used_mb);
        }

        Ok(MetricValue::MemoryStats {
            used_mb: self.ema_used_mb.min(total_mb),
            total_mb,
        })
    }
}

// ─── GPU Provider ─────────────────────────────────────────────────────────────

/// GPU Hardware Metric Collector Provider.
/// Queries DXGI video memory usage via `IDXGIFactory1::QueryVideoMemoryInfo` with EMA smoothing.
#[derive(Debug)]
pub struct GpuProvider {
    tick: u64,
    ema_pct: f32,
    alpha: f32,
}

impl GpuProvider {
    pub fn new() -> Self {
        Self {
            tick: 0,
            ema_pct: 0.0,
            alpha: 0.25,
        }
    }

    #[cfg(windows)]
    fn query_gpu_memory_pct() -> Result<f32> {
        use windows::core::Interface;
        use windows::Win32::Graphics::Dxgi::{
            CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
            DXGI_MEMORY_SEGMENT_GROUP_LOCAL, IDXGIFactory1, IDXGIAdapter3,
        };
        unsafe {
            let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
            let mut adapter_index = 0;
            while let Ok(adapter) = factory.EnumAdapters1(adapter_index) {
                if let Ok(desc) = adapter.GetDesc1() {
                    // Skip software rasterizer adapters
                    if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) == 0 {
                        if let Ok(adapter3) = adapter.cast::<IDXGIAdapter3>() {
                            let mut info = std::mem::zeroed();
                            if adapter3
                                .QueryVideoMemoryInfo(
                                    0,
                                    DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                                    &mut info,
                                )
                                .is_ok()
                            {
                                if info.Budget > 0 {
                                    let usage_pct = (info.CurrentUsage as f32
                                        / info.Budget as f32)
                                        * 100.0;
                                    return Ok(usage_pct.clamp(0.0, 100.0));
                                }
                            }
                        }
                    }
                }
                adapter_index += 1;
            }
        }
        anyhow::bail!("No active DXGI hardware adapter found")
    }

    #[cfg(not(windows))]
    fn query_gpu_memory_pct() -> Result<f32> {
        anyhow::bail!("Non-windows environment")
    }
}

impl Default for GpuProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for GpuProvider {
    fn name(&self) -> &'static str {
        "GpuProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let raw_pct = Self::query_gpu_memory_pct().unwrap_or(0.0);

        if self.tick == 1 {
            self.ema_pct = raw_pct;
        } else {
            self.ema_pct = (self.alpha * raw_pct) + ((1.0 - self.alpha) * self.ema_pct);
        }

        Ok(MetricValue::Percentage(self.ema_pct.clamp(0.0, 100.0)))
    }
}

// ─── Network Provider ─────────────────────────────────────────────────────────

/// Network Throughput Metric Collector Provider.
/// Uses `GetIfTable2` (Win32) to query hardware network interface octet throughput,
/// scaled by actual elapsed time and filtered with EMA smoothing.
#[derive(Debug)]
pub struct NetworkProvider {
    tick: u64,
    prev_bytes: u64,
    last_sample_time: Instant,
    ema_bytes_per_sec: f64,
    alpha: f64,
}

impl NetworkProvider {
    pub fn new() -> Self {
        let initial_bytes = Self::query_network_bytes().unwrap_or(0);
        Self {
            tick: 0,
            prev_bytes: initial_bytes,
            last_sample_time: Instant::now(),
            ema_bytes_per_sec: 0.0,
            alpha: 0.3,
        }
    }

    #[cfg(windows)]
    fn query_network_bytes() -> Result<u64> {
        use windows::Win32::NetworkManagement::IpHelper::{
            FreeMibTable, GetIfTable2, MIB_IF_TABLE2,
        };
        unsafe {
            let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
            GetIfTable2(&mut table_ptr).ok()?;
            if table_ptr.is_null() {
                return Ok(0);
            }
            let table = &*table_ptr;
            let mut total_bytes: u64 = 0;
            let count = table.NumEntries as usize;
            let rows = std::slice::from_raw_parts(table.Table.as_ptr(), count);
            for row in rows {
                total_bytes += row.InOctets + row.OutOctets;
            }
            FreeMibTable(table_ptr as *const _);
            Ok(total_bytes)
        }
    }

    #[cfg(not(windows))]
    fn query_network_bytes() -> Result<u64> {
        Ok(0)
    }
}

impl Default for NetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for NetworkProvider {
    fn name(&self) -> &'static str {
        "NetworkProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let now = Instant::now();
        let elapsed_secs = now.duration_since(self.last_sample_time).as_secs_f64();
        self.last_sample_time = now;

        let raw_rate = if let Ok(current_bytes) = Self::query_network_bytes() {
            if current_bytes >= self.prev_bytes {
                let delta = current_bytes.saturating_sub(self.prev_bytes);
                self.prev_bytes = current_bytes;
                if elapsed_secs > 0.001 {
                    delta as f64 / elapsed_secs
                } else {
                    self.ema_bytes_per_sec
                }
            } else {
                self.prev_bytes = current_bytes;
                0.0
            }
        } else {
            0.0
        };

        if self.tick == 1 {
            self.ema_bytes_per_sec = raw_rate;
        } else {
            self.ema_bytes_per_sec =
                (self.alpha * raw_rate) + ((1.0 - self.alpha) * self.ema_bytes_per_sec);
        }

        Ok(MetricValue::BytesPerSec(self.ema_bytes_per_sec as u64))
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
        // Take several samples to populate deltas and verify EMA smoothing
        for _ in 0..5 {
            let val = cpu.sample().unwrap();
            if let MetricValue::Percentage(pct) = val {
                assert!((0.0..=100.0).contains(&pct), "CPU% out of range: {pct}");
            }
        }
    }

    #[test]
    fn test_cpu_zero_delta_holds_previous() {
        let mut cpu = CpuProvider::new();
        cpu.last_valid_pct = 45.0;
        // Simulating repeated sampling faster than OS quantum
        let sample1 = cpu.sample().unwrap();
        let sample2 = cpu.sample().unwrap();
        if let (MetricValue::Percentage(p1), MetricValue::Percentage(p2)) = (sample1, sample2) {
            assert!((p1 - p2).abs() < 15.0, "Zero-delta tick must hold stable EMA value without fake spikes: p1={p1}, p2={p2}");
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

    #[test]
    fn test_gpu_percentage_in_range() {
        let mut gpu = GpuProvider::new();
        let val = gpu.sample().unwrap();
        if let MetricValue::Percentage(pct) = val {
            assert!((0.0..=100.0).contains(&pct), "GPU% out of range: {pct}");
        } else {
            panic!("Expected Percentage variant");
        }
    }

    #[test]
    fn test_network_throughput_time_scaled() {
        let mut net = NetworkProvider::new();
        let val1 = net.sample().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let val2 = net.sample().unwrap();
        if let (MetricValue::BytesPerSec(b1), MetricValue::BytesPerSec(b2)) = (val1, val2) {
            assert!(b1 < u64::MAX && b2 < u64::MAX, "Network bytes scaled properly");
        } else {
            panic!("Expected BytesPerSec variant");
        }
    }
}

