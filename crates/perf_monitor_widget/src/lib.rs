//! Aether Performance Monitor Widget
//!
//! A built-in Aether widget that reads live CPU%, GPU%, and RAM (used / free)
//! from the `SharedTelemetryCache` and emits a batch of `DrawCommand`s that
//! the DirectComposition renderer composites onto the desktop canvas.

pub mod renderer;

use anyhow::Result;
use system_providers::SharedTelemetryCache;
use tracing::info;
use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
use widget_sdk::render_config::RenderConfig;
use widget_sdk::rendering::{BatchRenderCanvas, RenderCanvas};

/// Performance Monitor Widget — implements the 6-pillar `WidgetLifecycle` SDK.
pub struct PerfMonitorWidget {
    state: WidgetState,
    cache: SharedTelemetryCache,
    tick_count: u64,
    config: RenderConfig,
}

impl PerfMonitorWidget {
    /// Creates a new `PerfMonitorWidget` bound to the shared telemetry cache.
    pub fn new(cache: SharedTelemetryCache) -> Self {
        Self {
            state: WidgetState::Unloaded,
            cache,
            tick_count: 0,
            config: RenderConfig::default(),
        }
    }

    pub fn set_config(&mut self, config: RenderConfig) {
        self.config = config;
    }

    pub fn config(&self) -> &RenderConfig {
        &self.config
    }
}

impl WidgetLifecycle for PerfMonitorWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("[PerfMonitorWidget] Widget loaded — bound to SharedTelemetryCache.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("[PerfMonitorWidget] Widget mounted to DirectComposition desktop canvas.");
        Ok(())
    }

    /// Called every update interval. Reads telemetry snapshot from shared cache
    /// (zero Windows API calls), builds draw commands, and logs metrics.
    fn on_update(&mut self, ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;
        let snap = self.cache.get_snapshot();

        // Build this frame's draw command list with active RenderConfig
        let mut canvas = BatchRenderCanvas::new();
        renderer::render_perf_card_with_config(&mut canvas, &snap, &self.config);

        let ram_total_gb = snap.memory_total_mb / 1024.0;
        let ram_used_gb  = snap.memory_used_mb  / 1024.0;
        let ram_free_gb  = ram_total_gb - ram_used_gb;
        let ram_pct      = if snap.memory_total_mb > 0.0 {
            (snap.memory_used_mb / snap.memory_total_mb) * 100.0
        } else {
            0.0
        };

        // Log the live stats every ~2 s (every 4 widget ticks at 500 ms each)
        if self.tick_count % 4 == 1 {
            info!(
                "[PerfMonitorWidget] Frame #{frame} \
                 | CPU: {cpu:.1}% \
                 | GPU: {gpu:.1}% \
                 | RAM: {used:.2}/{total:.2} GB ({pct:.0}% used, {free:.2} GB free) \
                 | DrawCmds: {cmds}",
                frame = ctx.frame_index,
                cpu   = snap.cpu_usage_pct,
                gpu   = snap.gpu_usage_pct,
                used  = ram_used_gb,
                total = ram_total_gb,
                pct   = ram_pct,
                free  = ram_free_gb,
                cmds  = canvas.commands().len(),
            );
        }

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("[PerfMonitorWidget] Widget unmounted from desktop canvas.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("[PerfMonitorWidget] Widget unloaded and resources freed.");
        Ok(())
    }

    fn state(&self) -> WidgetState {
        self.state
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use system_providers::{SharedTelemetryCache, TelemetrySnapshot};
    use std::collections::HashMap;

    fn make_snapshot(cpu: f32, gpu: f32, used_mb: f32, total_mb: f32) -> TelemetrySnapshot {
        TelemetrySnapshot {
            timestamp_ms: 1000,
            cpu_usage_pct: cpu,
            gpu_usage_pct: gpu,
            memory_used_mb: used_mb,
            memory_total_mb: total_mb,
            net_recv_bytes_per_sec: 0,
            net_sent_bytes_per_sec: 0,
            custom_metrics: HashMap::new(),
            ..TelemetrySnapshot::default()
        }
    }

    #[test]
    fn test_perf_widget_lifecycle() {
        let cache = SharedTelemetryCache::new();
        cache.update_snapshot(make_snapshot(42.0, 18.5, 8192.0, 16384.0));

        let mut widget = PerfMonitorWidget::new(cache);
        assert_eq!(widget.state(), WidgetState::Unloaded);

        widget.on_load().unwrap();
        assert_eq!(widget.state(), WidgetState::Loaded);

        widget.on_mount().unwrap();
        assert_eq!(widget.state(), WidgetState::Mounted);

        let ctx = TickContext { timestamp_ms: 1000, delta_time_ms: 500.0, frame_index: 1 };
        widget.on_update(&ctx).unwrap();
        assert_eq!(widget.tick_count, 1);

        widget.on_unmount().unwrap();
        assert_eq!(widget.state(), WidgetState::Unmounted);

        widget.on_unload().unwrap();
        assert_eq!(widget.state(), WidgetState::Unloaded);
    }

    #[test]
    fn test_renderer_produces_draw_commands() {
        use widget_sdk::rendering::BatchRenderCanvas;
        let snap = make_snapshot(65.0, 30.0, 12288.0, 16384.0);
        let mut canvas = BatchRenderCanvas::new();
        renderer::render_perf_card(&mut canvas, &snap);
        // Must produce multiple draw commands (background + bars + labels)
        assert!(canvas.commands().len() >= 10,
            "Expected ≥10 draw commands, got {}", canvas.commands().len());
    }

    #[test]
    fn test_perf_widget_empty_cache() {
        // Widget must survive an empty (zero) telemetry cache without panicking
        let cache = SharedTelemetryCache::new();
        let mut widget = PerfMonitorWidget::new(cache);
        widget.on_load().unwrap();
        widget.on_mount().unwrap();
        let ctx = TickContext { timestamp_ms: 0, delta_time_ms: 500.0, frame_index: 0 };
        // Should not panic on divide-by-zero (total_mb = 0 guard)
        assert!(widget.on_update(&ctx).is_ok());
    }
}
