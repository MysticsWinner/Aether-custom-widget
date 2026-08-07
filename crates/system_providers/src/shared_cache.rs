use ipc_protocol::MetricPayload;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

/// Immutable snapshot of all system telemetry metrics at a specific timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetrySnapshot {
    pub timestamp_ms: u64,
    pub cpu_usage_pct: f32,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub gpu_usage_pct: f32,
    pub net_recv_bytes_per_sec: u64,
    pub net_sent_bytes_per_sec: u64,
    // Process & Application Metrics
    pub open_apps_count: u32,
    pub browser_tabs_count: u32,
    pub audio_playing_apps_count: u32,
    pub gaming_apps_count: u32,
    pub dev_suite_apps_count: u32,
    pub other_apps_count: u32,
    // Power & Audio Metrics
    pub master_volume_pct: f32,
    pub is_muted: bool,
    pub battery_charge_pct: f32,
    pub battery_remaining_secs: u64,
    pub is_charging: bool,
    // Multi-GPU & Display Topology Metrics
    pub total_gpu_count: u32,
    pub integrated_gpu_count: u32,
    pub dedicated_gpu_count: u32,
    pub total_display_count: u32,
    pub external_display_count: u32,
    pub virtual_display_count: u32,
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for TelemetrySnapshot {
    fn default() -> Self {
        Self {
            timestamp_ms: 0,
            cpu_usage_pct: 0.0,
            memory_used_mb: 0.0,
            memory_total_mb: 16384.0,
            gpu_usage_pct: 0.0,
            net_recv_bytes_per_sec: 0,
            net_sent_bytes_per_sec: 0,
            open_apps_count: 5,
            browser_tabs_count: 12,
            audio_playing_apps_count: 1,
            gaming_apps_count: 0,
            dev_suite_apps_count: 2,
            other_apps_count: 2,
            master_volume_pct: 75.0,
            is_muted: false,
            battery_charge_pct: 85.0,
            battery_remaining_secs: 14400,
            is_charging: true,
            total_gpu_count: 2,
            integrated_gpu_count: 1,
            dedicated_gpu_count: 1,
            total_display_count: 2,
            external_display_count: 1,
            virtual_display_count: 0,
            custom_metrics: HashMap::new(),
        }
    }
}

impl From<MetricPayload> for TelemetrySnapshot {
    fn from(payload: MetricPayload) -> Self {
        Self {
            timestamp_ms: payload.timestamp_ms,
            cpu_usage_pct: payload.cpu_usage_pct,
            memory_used_mb: payload.memory_used_mb,
            memory_total_mb: payload.memory_total_mb,
            gpu_usage_pct: payload.gpu_usage_pct,
            net_recv_bytes_per_sec: payload.net_recv_bytes_per_sec,
            net_sent_bytes_per_sec: payload.net_sent_bytes_per_sec,
            open_apps_count: payload.open_apps_count,
            browser_tabs_count: payload.browser_tabs_count,
            audio_playing_apps_count: payload.audio_playing_apps_count,
            gaming_apps_count: payload.gaming_apps_count,
            dev_suite_apps_count: payload.dev_suite_apps_count,
            other_apps_count: payload.other_apps_count,
            master_volume_pct: payload.master_volume_pct,
            is_muted: payload.is_muted,
            battery_charge_pct: payload.battery_charge_pct,
            battery_remaining_secs: payload.battery_remaining_secs,
            is_charging: payload.is_charging,
            total_gpu_count: payload.total_gpu_count,
            integrated_gpu_count: payload.integrated_gpu_count,
            dedicated_gpu_count: payload.dedicated_gpu_count,
            total_display_count: payload.total_display_count,
            external_display_count: payload.external_display_count,
            virtual_display_count: payload.virtual_display_count,
            custom_metrics: HashMap::new(),
        }
    }
}

/// Central Shared Telemetry Cache.
/// Implements "Collect Once, Publish Everywhere" - consumers read from cache; widgets never query Windows APIs directly.
#[derive(Clone)]
pub struct SharedTelemetryCache {
    snapshot: Arc<RwLock<TelemetrySnapshot>>,
    update_count: Arc<RwLock<u64>>,
}

impl SharedTelemetryCache {
    /// Creates a new empty `SharedTelemetryCache`.
    pub fn new() -> Self {
        Self {
            snapshot: Arc::new(RwLock::new(TelemetrySnapshot::default())),
            update_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Atomically updates the cached snapshot (called strictly by TelemetryService).
    pub fn update_snapshot(&self, new_snapshot: TelemetrySnapshot) {
        debug!(
            "Shared Cache update tick #{}: CPU={:.1}%, Memory={:.1}MB",
            new_snapshot.timestamp_ms, new_snapshot.cpu_usage_pct, new_snapshot.memory_used_mb
        );

        if let Ok(mut lock) = self.snapshot.write() {
            *lock = new_snapshot;
        }
        if let Ok(mut count) = self.update_count.write() {
            *count += 1;
        }
    }

    /// Returns a zero-copy clone of the current immutable `TelemetrySnapshot`.
    pub fn get_snapshot(&self) -> TelemetrySnapshot {
        self.snapshot
            .read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Returns the current CPU usage percentage from shared cache.
    pub fn get_cpu_pct(&self) -> f32 {
        self.snapshot.read().map(|s| s.cpu_usage_pct).unwrap_or(0.0)
    }

    /// Returns the current used memory in MB from shared cache.
    pub fn get_memory_used_mb(&self) -> f32 {
        self.snapshot.read().map(|s| s.memory_used_mb).unwrap_or(0.0)
    }

    /// Returns total cache update count.
    pub fn update_count(&self) -> u64 {
        self.update_count.read().map(|g| *g).unwrap_or(0)
    }
}

impl Default for SharedTelemetryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_cache_collect_once_publish_everywhere() {
        let cache = SharedTelemetryCache::new();
        assert_eq!(cache.get_cpu_pct(), 0.0);
        assert_eq!(cache.update_count(), 0);

        let snapshot = TelemetrySnapshot {
            timestamp_ms: 1000,
            cpu_usage_pct: 45.2,
            memory_used_mb: 4096.0,
            memory_total_mb: 16384.0,
            gpu_usage_pct: 18.0,
            net_recv_bytes_per_sec: 2048,
            net_sent_bytes_per_sec: 512,
            custom_metrics: HashMap::new(),
            ..TelemetrySnapshot::default()
        };

        cache.update_snapshot(snapshot.clone());

        assert_eq!(cache.get_cpu_pct(), 45.2);
        assert_eq!(cache.get_memory_used_mb(), 4096.0);
        assert_eq!(cache.get_snapshot(), snapshot);
        assert_eq!(cache.update_count(), 1);
    }
}
