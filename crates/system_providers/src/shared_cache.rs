use crate::providers::cpu_topology::CpuTopologyTelemetry;
use crate::providers::crypto_financial::CryptoAssetTelemetry;
use crate::providers::gpu_d3dkmt::GpuTelemetry;
use crate::providers::network_diagnostics::NetworkDiagnosticsTelemetry;
use crate::providers::wasapi_audio::{AudioSpectrumTelemetry, MediaPlaybackTelemetry};
use ipc_protocol::MetricPayload;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
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
    // Extended Phase 1 Deep Hardware & Audio Telemetry
    pub gpu_telemetry: GpuTelemetry,
    pub cpu_topology: CpuTopologyTelemetry,
    pub audio_spectrum: AudioSpectrumTelemetry,
    pub media_playback: MediaPlaybackTelemetry,
    // Extended Financial & Network Telemetry
    pub crypto_assets: Vec<CryptoAssetTelemetry>,
    pub network_diagnostics: NetworkDiagnosticsTelemetry,
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
            gpu_telemetry: GpuTelemetry::default(),
            cpu_topology: CpuTopologyTelemetry::default(),
            audio_spectrum: AudioSpectrumTelemetry::default(),
            media_playback: MediaPlaybackTelemetry::default(),
            crypto_assets: Vec::new(),
            network_diagnostics: NetworkDiagnosticsTelemetry::default(),
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
            gpu_telemetry: GpuTelemetry::default(),
            cpu_topology: CpuTopologyTelemetry::default(),
            audio_spectrum: AudioSpectrumTelemetry::default(),
            media_playback: MediaPlaybackTelemetry::default(),
            crypto_assets: Vec::new(),
            network_diagnostics: NetworkDiagnosticsTelemetry::default(),
            custom_metrics: HashMap::new(),
        }
    }
}

impl From<TelemetrySnapshot> for MetricPayload {
    fn from(snap: TelemetrySnapshot) -> Self {
        Self {
            timestamp_ms: snap.timestamp_ms,
            cpu_usage_pct: snap.cpu_usage_pct,
            memory_used_mb: snap.memory_used_mb,
            memory_total_mb: snap.memory_total_mb,
            gpu_usage_pct: snap.gpu_usage_pct,
            net_recv_bytes_per_sec: snap.net_recv_bytes_per_sec,
            net_sent_bytes_per_sec: snap.net_sent_bytes_per_sec,
            open_apps_count: snap.open_apps_count,
            browser_tabs_count: snap.browser_tabs_count,
            audio_playing_apps_count: snap.audio_playing_apps_count,
            gaming_apps_count: snap.gaming_apps_count,
            dev_suite_apps_count: snap.dev_suite_apps_count,
            other_apps_count: snap.other_apps_count,
            master_volume_pct: snap.master_volume_pct,
            is_muted: snap.is_muted,
            battery_charge_pct: snap.battery_charge_pct,
            battery_remaining_secs: snap.battery_remaining_secs,
            is_charging: snap.is_charging,
            total_gpu_count: snap.total_gpu_count,
            integrated_gpu_count: snap.integrated_gpu_count,
            dedicated_gpu_count: snap.dedicated_gpu_count,
            total_display_count: snap.total_display_count,
            external_display_count: snap.external_display_count,
            virtual_display_count: snap.virtual_display_count,
        }
    }
}

impl From<&TelemetrySnapshot> for MetricPayload {
    fn from(snap: &TelemetrySnapshot) -> Self {
        Self::from(snap.clone())
    }
}

/// Thread-safe in-memory telemetry publication store implementing the "Collect Once, Publish Everywhere" principle.
///
/// Architecture Guarantee:
/// - Scalar metric queries (`get_cpu_pct`, `get_memory_used_mb`, `get_gpu_pct`, etc.) are WAIT-FREE,
///   zero-allocation reads directly backed by cacheline-friendly `AtomicU32` and `AtomicU64` registers.
/// - Readers NEVER acquire a lock or block the telemetry producer thread during scalar metric queries.
/// - Full snapshots are published with release-acquire sequence numbers for stale snapshot detection.
#[derive(Debug, Clone)]
pub struct SharedTelemetryCache {
    snapshot: Arc<RwLock<TelemetrySnapshot>>,
    update_count: Arc<AtomicU64>,
    sequence: Arc<AtomicU64>,
    timestamp_ms: Arc<AtomicU64>,
    cpu_pct_bits: Arc<AtomicU32>,
    gpu_pct_bits: Arc<AtomicU32>,
    memory_used_mb_bits: Arc<AtomicU32>,
    memory_total_mb_bits: Arc<AtomicU32>,
    net_recv_bytes_per_sec: Arc<AtomicU64>,
    net_sent_bytes_per_sec: Arc<AtomicU64>,
}

impl SharedTelemetryCache {
    /// Creates a new `SharedTelemetryCache` initialized with default metrics.
    pub fn new() -> Self {
        Self {
            snapshot: Arc::new(RwLock::new(TelemetrySnapshot::default())),
            update_count: Arc::new(AtomicU64::new(0)),
            sequence: Arc::new(AtomicU64::new(0)),
            timestamp_ms: Arc::new(AtomicU64::new(0)),
            cpu_pct_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            gpu_pct_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            memory_used_mb_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            memory_total_mb_bits: Arc::new(AtomicU32::new(16384.0f32.to_bits())),
            net_recv_bytes_per_sec: Arc::new(AtomicU64::new(0)),
            net_sent_bytes_per_sec: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Atomically updates the entire telemetry snapshot.
    /// Scalar atomic registers are updated first with Release semantics, followed by the sequence counter.
    pub fn update_snapshot(&self, new_snapshot: TelemetrySnapshot) {
        // 1. Publish wait-free atomic scalar registers
        self.cpu_pct_bits.store(new_snapshot.cpu_usage_pct.to_bits(), Ordering::Release);
        self.gpu_pct_bits.store(new_snapshot.gpu_usage_pct.to_bits(), Ordering::Release);
        self.memory_used_mb_bits.store(new_snapshot.memory_used_mb.to_bits(), Ordering::Release);
        self.memory_total_mb_bits.store(new_snapshot.memory_total_mb.to_bits(), Ordering::Release);
        self.net_recv_bytes_per_sec.store(new_snapshot.net_recv_bytes_per_sec, Ordering::Release);
        self.net_sent_bytes_per_sec.store(new_snapshot.net_sent_bytes_per_sec, Ordering::Release);
        self.timestamp_ms.store(new_snapshot.timestamp_ms, Ordering::Release);

        // 2. Increment monotonic sequence counter
        self.sequence.fetch_add(1, Ordering::Release);
        self.update_count.fetch_add(1, Ordering::Release);

        // 3. Update full composite snapshot
        if let Ok(mut snap) = self.snapshot.write() {
            *snap = new_snapshot;
        }
        debug!("SharedTelemetryCache snapshot updated successfully.");
    }

    /// Retrieves an immutable copy of the current `TelemetrySnapshot`.
    pub fn get_snapshot(&self) -> TelemetrySnapshot {
        self.snapshot.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Returns the latest CPU usage percentage from shared cache (wait-free, zero-allocation).
    #[inline]
    pub fn get_cpu_pct(&self) -> f32 {
        f32::from_bits(self.cpu_pct_bits.load(Ordering::Acquire))
    }

    /// Returns the latest GPU usage percentage from shared cache (wait-free, zero-allocation).
    #[inline]
    pub fn get_gpu_pct(&self) -> f32 {
        f32::from_bits(self.gpu_pct_bits.load(Ordering::Acquire))
    }

    /// Returns the latest RAM used in megabytes from shared cache (wait-free, zero-allocation).
    #[inline]
    pub fn get_memory_used_mb(&self) -> f32 {
        f32::from_bits(self.memory_used_mb_bits.load(Ordering::Acquire))
    }

    /// Returns the latest RAM total in megabytes from shared cache (wait-free, zero-allocation).
    #[inline]
    pub fn get_memory_total_mb(&self) -> f32 {
        f32::from_bits(self.memory_total_mb_bits.load(Ordering::Acquire))
    }

    /// Returns network received bytes per second (wait-free).
    #[inline]
    pub fn get_net_recv_bytes_per_sec(&self) -> u64 {
        self.net_recv_bytes_per_sec.load(Ordering::Acquire)
    }

    /// Returns network sent bytes per second (wait-free).
    #[inline]
    pub fn get_net_sent_bytes_per_sec(&self) -> u64 {
        self.net_sent_bytes_per_sec.load(Ordering::Acquire)
    }

    /// Returns snapshot timestamp in milliseconds epoch (wait-free).
    #[inline]
    pub fn timestamp_ms(&self) -> u64 {
        self.timestamp_ms.load(Ordering::Acquire)
    }

    /// Returns monotonic sequence number incremented upon every snapshot update (wait-free).
    #[inline]
    pub fn sequence(&self) -> u64 {
        self.sequence.load(Ordering::Acquire)
    }

    /// Checks if the cached telemetry snapshot is stale relative to `current_time_ms`.
    pub fn is_stale(&self, current_time_ms: u64, max_age_ms: u64) -> bool {
        let ts = self.timestamp_ms.load(Ordering::Acquire);
        if ts == 0 {
            return false;
        }
        current_time_ms.saturating_sub(ts) > max_age_ms
    }

    /// Returns the latest dedicated GPU telemetry from shared cache.
    pub fn get_gpu_telemetry(&self) -> GpuTelemetry {
        self.snapshot.read().map(|s| s.gpu_telemetry.clone()).unwrap_or_default()
    }

    /// Returns the latest CPU topology telemetry from shared cache.
    pub fn get_cpu_topology(&self) -> CpuTopologyTelemetry {
        self.snapshot.read().map(|s| s.cpu_topology.clone()).unwrap_or_default()
    }

    /// Returns the 16-band audio FFT frequency spectrum from shared cache.
    pub fn get_audio_spectrum(&self) -> AudioSpectrumTelemetry {
        self.snapshot.read().map(|s| s.audio_spectrum.clone()).unwrap_or_default()
    }

    /// Returns the active media playback metadata from shared cache.
    pub fn get_media_playback(&self) -> MediaPlaybackTelemetry {
        self.snapshot.read().map(|s| s.media_playback.clone()).unwrap_or_default()
    }

    /// Returns the live financial and crypto assets from shared cache.
    pub fn get_crypto_assets(&self) -> Vec<CryptoAssetTelemetry> {
        self.snapshot.read().map(|s| s.crypto_assets.clone()).unwrap_or_default()
    }

    /// Returns deep network diagnostics from shared cache.
    pub fn get_network_diagnostics(&self) -> NetworkDiagnosticsTelemetry {
        self.snapshot.read().map(|s| s.network_diagnostics.clone()).unwrap_or_default()
    }

    /// Returns total cache update count (wait-free).
    #[inline]
    pub fn update_count(&self) -> u64 {
        self.update_count.load(Ordering::Acquire)
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
    use std::sync::atomic::AtomicBool;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_shared_cache_collect_once_publish_everywhere() {
        let cache = SharedTelemetryCache::new();
        assert_eq!(cache.get_cpu_pct(), 0.0);
        assert_eq!(cache.update_count(), 0);
        assert_eq!(cache.sequence(), 0);

        let mut snapshot = TelemetrySnapshot {
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
        snapshot.gpu_telemetry.vram_dedicated_used_mb = 2048.0;
        snapshot.media_playback.title = "Synthwave".to_string();
        snapshot.crypto_assets = vec![CryptoAssetTelemetry::default()];
        snapshot.network_diagnostics.ping_latency_ms = 9;

        cache.update_snapshot(snapshot.clone());

        assert_eq!(cache.get_cpu_pct(), 45.2);
        assert_eq!(cache.get_memory_used_mb(), 4096.0);
        assert_eq!(cache.get_gpu_telemetry().vram_dedicated_used_mb, 2048.0);
        assert_eq!(cache.get_media_playback().title, "Synthwave");
        assert_eq!(cache.get_crypto_assets().len(), 1);
        assert_eq!(cache.get_network_diagnostics().ping_latency_ms, 9);
        assert_eq!(cache.get_snapshot(), snapshot);
        assert_eq!(cache.update_count(), 1);
        assert_eq!(cache.sequence(), 1);
        assert_eq!(cache.timestamp_ms(), 1000);
        assert!(!cache.is_stale(1050, 100));
        assert!(cache.is_stale(1200, 100));
    }

    #[test]
    fn test_shared_cache_concurrent_readers_and_writer() {
        let cache = Arc::new(SharedTelemetryCache::new());
        let running = Arc::new(AtomicBool::new(true));

        let mut handles = Vec::new();

        // Spawn 4 reader threads querying scalar metrics at high frequency
        for _ in 0..4 {
            let c = cache.clone();
            let r = running.clone();
            handles.push(thread::spawn(move || {
                let mut reads = 0;
                while r.load(Ordering::Relaxed) {
                    let cpu = c.get_cpu_pct();
                    let mem = c.get_memory_used_mb();
                    let _seq = c.sequence();
                    assert!(cpu >= 0.0 && cpu <= 100.0);
                    assert!(mem >= 0.0);
                    reads += 1;
                }
                reads
            }));
        }

        // Writer thread publishing snapshots
        let c_writer = cache.clone();
        let writer_handle = thread::spawn(move || {
            for i in 1..=100 {
                let mut snap = TelemetrySnapshot::default();
                snap.cpu_usage_pct = (i % 100) as f32;
                snap.memory_used_mb = (i * 10) as f32;
                snap.timestamp_ms = i as u64 * 10;
                c_writer.update_snapshot(snap);
                thread::sleep(Duration::from_micros(200));
            }
        });

        writer_handle.join().unwrap();
        running.store(false, Ordering::Relaxed);

        let total_reads: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert!(total_reads > 1000, "Readers must complete thousands of wait-free reads");
        assert_eq!(cache.sequence(), 100);
    }
}
