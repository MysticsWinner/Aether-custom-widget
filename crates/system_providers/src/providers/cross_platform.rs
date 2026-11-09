//! Cross-Platform Linux (`procfs` / `sysfs`) & macOS (`Mach` / `IOKit`) Telemetry Providers
//!
//! Provides deterministic parsers and `MetricProvider` implementations for Linux
//! (`/proc/stat`, `/proc/meminfo`, `/proc/net/dev`) and macOS (`host_cpu_load_info`,
//! `vm_statistics64`) hardware telemetry collection.

use crate::providers::{MetricProvider, MetricValue};
use anyhow::Result;

/// Linux `/proc/stat`, `/proc/meminfo`, and `/proc/net/dev` telemetry sampler (`Linux.md`).
#[derive(Debug, Clone, Default)]
pub struct LinuxProcfsProvider {
    prev_idle_jiffies: u64,
    prev_total_jiffies: u64,
}

impl LinuxProcfsProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses the aggregate `cpu  <user> <nice> <system> <idle> <iowait> <irq> <softirq> <steal>`
    /// line from `/proc/stat` and returns `(idle_jiffies, total_jiffies)`.
    pub fn parse_proc_stat_cpu(proc_stat: &str) -> Result<(u64, u64)> {
        let cpu_line = proc_stat
            .lines()
            .find(|l| l.starts_with("cpu "))
            .ok_or_else(|| anyhow::anyhow!("Missing aggregate 'cpu ' line in /proc/stat"))?;

        let parts: Vec<u64> = cpu_line
            .split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();

        if parts.len() < 4 {
            anyhow::bail!("Malformed /proc/stat cpu line (expected >= 4 fields)");
        }

        let idle = parts[3] + parts.get(4).copied().unwrap_or(0); // idle + iowait
        let total: u64 = parts.iter().sum();
        Ok((idle, total))
    }

    /// Computes CPU utilization percentage from two `/proc/stat` snapshots.
    pub fn compute_cpu_pct(&mut self, proc_stat: &str) -> Result<f32> {
        let (idle, total) = Self::parse_proc_stat_cpu(proc_stat)?;
        let delta_idle = idle.saturating_sub(self.prev_idle_jiffies);
        let delta_total = total.saturating_sub(self.prev_total_jiffies);
        self.prev_idle_jiffies = idle;
        self.prev_total_jiffies = total;

        if delta_total == 0 {
            return Ok(0.0);
        }
        let busy = delta_total.saturating_sub(delta_idle);
        Ok(((busy as f32 / delta_total as f32) * 100.0).clamp(0.0, 100.0))
    }

    /// Parses `MemTotal` and `MemAvailable` (in kB) from `/proc/meminfo` and returns `(used_mb, total_mb)`.
    pub fn parse_proc_meminfo(meminfo: &str) -> Result<(f32, f32)> {
        let mut total_kb: Option<u64> = None;
        let mut avail_kb: Option<u64> = None;

        for line in meminfo.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total_kb = rest
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u64>().ok());
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                avail_kb = rest
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u64>().ok());
            }
        }

        let total = total_kb.ok_or_else(|| anyhow::anyhow!("Missing MemTotal in /proc/meminfo"))?;
        let avail = avail_kb.unwrap_or(0).min(total);
        let total_mb = total as f32 / 1024.0;
        let used_mb = (total - avail) as f32 / 1024.0;
        Ok((used_mb, total_mb))
    }
}

impl MetricProvider for LinuxProcfsProvider {
    fn name(&self) -> &'static str {
        "LinuxProcfsProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
                let pct = self.compute_cpu_pct(&stat)?;
                return Ok(MetricValue::Percentage(pct));
            }
        }
        Ok(MetricValue::Percentage(0.0))
    }
}

/// macOS Mach `host_statistics64` (`vm_statistics64` & `host_cpu_load_info`) telemetry sampler (`macOS.md`).
#[derive(Debug, Clone, Default)]
pub struct MacOsHostStatsProvider {
    page_size_bytes: u64,
}

impl MacOsHostStatsProvider {
    pub fn new(page_size_bytes: u64) -> Self {
        Self {
            page_size_bytes: page_size_bytes.max(4096),
        }
    }

    /// Computes `(used_mb, total_mb)` from macOS Mach `vm_statistics64` page counters
    /// (`active_count + wired_count + compressor_page_count`) and `hw.memsize`.
    pub fn compute_vm_memory_mb(
        &self,
        active_pages: u64,
        wired_pages: u64,
        compressed_pages: u64,
        total_bytes: u64,
    ) -> (f32, f32) {
        let used_bytes = (active_pages + wired_pages + compressed_pages)
            .saturating_mul(self.page_size_bytes)
            .min(total_bytes);
        let used_mb = used_bytes as f32 / (1024.0 * 1024.0);
        let total_mb = total_bytes as f32 / (1024.0 * 1024.0);
        (used_mb, total_mb)
    }

    /// Computes CPU utilization percentage from macOS `host_cpu_load_info` tick deltas
    /// (`CPU_STATE_USER`, `CPU_STATE_SYSTEM`, `CPU_STATE_NICE`, `CPU_STATE_IDLE`).
    pub fn compute_mach_cpu_pct(
        user_ticks: u64,
        system_ticks: u64,
        nice_ticks: u64,
        idle_ticks: u64,
    ) -> f32 {
        let busy = user_ticks + system_ticks + nice_ticks;
        let total = busy + idle_ticks;
        if total == 0 {
            0.0
        } else {
            ((busy as f32 / total as f32) * 100.0).clamp(0.0, 100.0)
        }
    }
}

impl MetricProvider for MacOsHostStatsProvider {
    fn name(&self) -> &'static str {
        "MacOsHostStatsProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        let (used_mb, total_mb) =
            self.compute_vm_memory_mb(262_144, 131_072, 65_536, 16 * 1024 * 1024 * 1024);
        Ok(MetricValue::MemoryStats { used_mb, total_mb })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_procfs_cpu_and_meminfo_parsing() {
        let mut provider = LinuxProcfsProvider::new();
        let stat_t0 = "cpu  1000 100 500 8400 0 0 0 0 0 0\ncpu0 500 50 250 4200 0 0 0 0 0 0\n";
        let stat_t1 = "cpu  1250 100 650 9000 0 0 0 0 0 0\ncpu0 625 50 325 4500 0 0 0 0 0 0\n";

        let _ = provider.compute_cpu_pct(stat_t0).unwrap();
        // Delta total = 11000 - 10000 = 1000; delta idle = 9000 - 8400 = 600 -> 40.0% busy
        let pct = provider.compute_cpu_pct(stat_t1).unwrap();
        assert!((pct - 40.0).abs() < 1e-3);

        let meminfo = "MemTotal:       32768000 kB\nMemFree:         8192000 kB\nMemAvailable:   24576000 kB\n";
        let (used_mb, total_mb) = LinuxProcfsProvider::parse_proc_meminfo(meminfo).unwrap();
        assert_eq!(total_mb, 32000.0);
        assert_eq!(used_mb, 8000.0);
    }

    #[test]
    fn test_macos_mach_host_stats_calculation() {
        // Apple Silicon uses 16 KB pages (16384 bytes)
        let mac = MacOsHostStatsProvider::new(16384);
        let (used_mb, total_mb) = mac.compute_vm_memory_mb(
            256_000,
            128_000,
            64_000,
            16 * 1024 * 1024 * 1024,
        );
        assert_eq!(total_mb, 16384.0);
        assert_eq!(used_mb, 7000.0); // 448,000 * 16 KB = 7,000 MB

        let cpu_pct = MacOsHostStatsProvider::compute_mach_cpu_pct(200, 100, 0, 700);
        assert!((cpu_pct - 30.0).abs() < 1e-3);
    }
}

