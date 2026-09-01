//! Aether Hardware Pro Sensor Matrix Widget
//!
//! A built-in high-performance monitoring widget that visualizes dedicated GPU VRAM,
//! 3D/Video/Copy engine load, Intel/AMD hybrid P-core vs E-core load matrix,
//! historical spline load graphs, and thermal throttling alerts from `SharedTelemetryCache`.

use anyhow::Result;
use system_providers::SharedTelemetryCache;
use theme_engine::{MaterialSpec, MaterialType};
use tracing::info;
use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
use widget_sdk::perf_budget::PerformanceBudget;
use widget_sdk::reactive::{Computed, Signal};
use widget_sdk::render_config::RenderConfig;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas, RenderEffect};

/// Hardware Pro Sensor Matrix Widget — implements the 6-pillar `WidgetLifecycle` SDK.
pub struct HardwareProWidget {
    state: WidgetState,
    cache: SharedTelemetryCache,
    gpu_3d_signal: Signal<f32>,
    cpu_signal: Signal<f32>,
    vram_used_signal: Signal<f32>,
    is_high_load: Computed<bool>,
    history_points: Vec<(f32, f32)>,
    material: MaterialSpec,
    budget: PerformanceBudget,
    tick_count: u64,
    config: RenderConfig,
}

impl HardwareProWidget {
    /// Creates a new `HardwareProWidget` bound to the shared telemetry cache.
    pub fn new(cache: SharedTelemetryCache) -> Self {
        let is_high_load = Computed::new("is_high_load", || false);

        Self {
            state: WidgetState::Unloaded,
            cache,
            gpu_3d_signal: Signal::new("gpu.3d", 0.0),
            cpu_signal: Signal::new("cpu.overall", 0.0),
            vram_used_signal: Signal::new("gpu.vram_used", 0.0),
            is_high_load,
            history_points: Vec::with_capacity(30),
            material: MaterialSpec {
                material_type: MaterialType::Acrylic,
                tint_color: "#0D1117".to_string(),
                tint_opacity: 0.92,
                blur_radius: 30.0,
                ..Default::default()
            },
            budget: PerformanceBudget {
                target_cpu_pct: 0.08,
                target_memory_mb: 16.0,
                target_fps: 60,
                material_cost: "medium".to_string(),
                animation_cost: "low".to_string(),
            },
            tick_count: 0,
            config: RenderConfig::default(),
        }
    }
}

impl WidgetLifecycle for HardwareProWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("HardwareProWidget loaded.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("HardwareProWidget mounted.");
        Ok(())
    }

    fn on_update(&mut self, _ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;
        let snap = self.cache.get_snapshot();
        let gpu = snap.gpu_telemetry;
        let cpu_topo = snap.cpu_topology;
        let cpu_pct = snap.cpu_usage_pct;

        self.gpu_3d_signal.set(gpu.utilization_3d_pct);
        self.cpu_signal.set(cpu_pct);
        self.vram_used_signal.set(gpu.vram_dedicated_used_mb);

        // Update historical spline graph points
        let graph_width = 380.0;
        let graph_height = 40.0;
        let max_samples = 25;
        let new_y = 190.0 - (gpu.utilization_3d_pct / 100.0 * graph_height);
        let new_x = 20.0 + (self.history_points.len() as f32 / max_samples as f32 * graph_width);

        if self.history_points.len() >= max_samples {
            self.history_points.remove(0);
            for (idx, pt) in self.history_points.iter_mut().enumerate() {
                pt.0 = 20.0 + (idx as f32 / max_samples as f32 * graph_width);
            }
        }
        self.history_points.push((new_x, new_y));

        let mut canvas = BatchRenderCanvas::new();

        // 1. Background Card with Blur
        canvas.draw_effect(
            RenderEffect::GaussianBlur { radius: 12.0 },
            RectF::new(0.0, 0.0, 420.0, 240.0),
        );
        canvas.draw_rect(
            RectF::new(0.0, 0.0, 420.0, 240.0),
            Color::rgba(0.05, 0.07, 0.10, 0.94),
            16.0,
        );

        // 2. GPU Section Header & Clocks
        canvas.draw_text(
            &gpu.adapter_name,
            "Segoe UI",
            14.0,
            RectF::new(20.0, 16.0, 260.0, 20.0),
            Color::rgba(0.95, 0.95, 1.0, 1.0),
        );
        canvas.draw_text(
            &format!("{:.0}°C • {} MHz • {:.0}% Fan", gpu.temperature_c, gpu.clock_core_mhz, gpu.fan_speed_pct),
            "Segoe UI",
            11.0,
            RectF::new(280.0, 18.0, 120.0, 18.0),
            Color::rgba(0.0, 0.90, 0.80, 0.9), // Cyan/Mint accent
        );

        // 3. Dedicated VRAM Bar
        let vram_pct = (gpu.vram_dedicated_used_mb / gpu.vram_dedicated_total_mb.max(1.0)).clamp(0.0, 1.0);
        canvas.draw_text(
            &format!("VRAM: {:.0} MB / {:.0} MB ({:.0}%)", gpu.vram_dedicated_used_mb, gpu.vram_dedicated_total_mb, vram_pct * 100.0),
            "Segoe UI",
            11.0,
            RectF::new(20.0, 40.0, 380.0, 16.0),
            Color::rgba(0.8, 0.8, 0.9, 0.8),
        );
        canvas.draw_rect(RectF::new(20.0, 58.0, 380.0, 6.0), Color::rgba(0.2, 0.2, 0.3, 0.5), 3.0);
        canvas.draw_rect(RectF::new(20.0, 58.0, 380.0 * vram_pct, 6.0), Color::rgb(0.0, 0.85, 1.0), 3.0);

        // 4. CPU Multi-Core Topology Matrix (P-Cores vs E-Cores)
        canvas.draw_text(
            &format!("CPU Cores: {} Logical ({} P-Cores, {} E-Cores) • Overall {:.1}%", cpu_topo.logical_core_count, cpu_topo.p_core_count, cpu_topo.e_core_count, cpu_pct),
            "Segoe UI",
            11.0,
            RectF::new(20.0, 74.0, 380.0, 16.0),
            Color::rgba(0.85, 0.85, 0.95, 0.9),
        );

        // Render mini core load tiles
        let num_cores = cpu_topo.per_core_usage_pct.len().min(16);
        let core_w = 20.0;
        let core_gap = 4.0;
        for (i, &load) in cpu_topo.per_core_usage_pct.iter().take(num_cores).enumerate() {
            let core_x = 20.0 + (i as f32 * (core_w + core_gap));
            let is_p_core = (i as u32) < cpu_topo.p_core_count;
            let bar_color = if is_p_core {
                Color::rgba(0.0, 0.8, 1.0, (load / 100.0).max(0.3)) // P-Core: Cyan
            } else {
                Color::rgba(0.7, 0.4, 1.0, (load / 100.0).max(0.3)) // E-Core: Purple
            };

            canvas.draw_rect(RectF::new(core_x, 94.0, core_w, 24.0), Color::rgba(0.15, 0.15, 0.22, 0.6), 4.0);
            canvas.draw_rect(RectF::new(core_x, 94.0 + (24.0 * (1.0 - load / 100.0)), core_w, 24.0 * (load / 100.0)), bar_color, 4.0);
        }

        // 5. Historical Spline Load Area Graph
        if self.history_points.len() > 1 {
            canvas.draw_spline_area(
                self.history_points.clone(),
                Color::rgba(0.0, 0.85, 1.0, 0.20),
                Color::rgb(0.0, 0.85, 1.0),
                1.5,
            );
        }

        // 6. SVG Vector Icon
        canvas.draw_path("M 0 0 L 10 10 L 0 20 Z", Some(Color::rgb(0.0, 0.9, 1.0)), None, 1.0);

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("HardwareProWidget unmounted.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("HardwareProWidget unloaded.");
        Ok(())
    }

    fn state(&self) -> WidgetState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use system_providers::TelemetrySnapshot;

    #[test]
    fn test_hardware_pro_lifecycle() {
        let cache = SharedTelemetryCache::new();
        let mut widget = HardwareProWidget::new(cache);
        assert_eq!(widget.state(), WidgetState::Unloaded);

        assert!(widget.on_load().is_ok());
        assert_eq!(widget.state(), WidgetState::Loaded);

        assert!(widget.on_mount().is_ok());
        assert_eq!(widget.state(), WidgetState::Mounted);

        let ctx = TickContext {
            tick_number: 1,
            delta_ms: 16.6,
            system_time_ms: 1000,
        };
        assert!(widget.on_update(&ctx).is_ok());

        assert!(widget.on_unmount().is_ok());
        assert_eq!(widget.state(), WidgetState::Unmounted);

        assert!(widget.on_unload().is_ok());
        assert_eq!(widget.state(), WidgetState::Unloaded);
    }

    #[test]
    fn test_hardware_pro_renders_with_gpu_and_cpu_topology() {
        let cache = SharedTelemetryCache::new();
        let mut snap = TelemetrySnapshot::default();
        snap.gpu_telemetry.adapter_name = "NVIDIA GeForce RTX 4090".to_string();
        snap.gpu_telemetry.vram_dedicated_used_mb = 6144.0;
        snap.gpu_telemetry.vram_dedicated_total_mb = 24576.0;
        snap.gpu_telemetry.utilization_3d_pct = 68.0;
        snap.cpu_topology.physical_core_count = 16;
        snap.cpu_topology.logical_core_count = 24;
        snap.cpu_topology.p_core_count = 8;
        snap.cpu_topology.e_core_count = 16;
        snap.cpu_topology.per_core_usage_pct = vec![55.0; 24];
        cache.update_snapshot(snap);

        let mut widget = HardwareProWidget::new(cache);
        widget.on_load().unwrap();
        widget.on_mount().unwrap();

        let ctx = TickContext {
            tick_number: 1,
            delta_ms: 16.6,
            system_time_ms: 1000,
        };
        assert!(widget.on_update(&ctx).is_ok());
        assert!(!widget.history_points.is_empty());
    }
}
