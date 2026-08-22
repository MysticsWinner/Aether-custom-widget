//! Aether Weather & Environmental Widget Plugin
//!
//! Provides a built-in Weather & Forecast widget skin implementing `WidgetLifecycle`
//! and emitting Direct2D draw commands for temperature, humidity, wind, and conditions.

use anyhow::Result;
use system_providers::SharedTelemetryCache;
use theme_engine::{MaterialSpec, MaterialType};
use tracing::info;
use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
use widget_sdk::perf_budget::PerformanceBudget;
use widget_sdk::reactive::Signal;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas};

pub struct WeatherWidget {
    state: WidgetState,
    cache: SharedTelemetryCache,
    temp_signal: Signal<f32>,
    humidity_signal: Signal<f32>,
    material: MaterialSpec,
    budget: PerformanceBudget,
    tick_count: u64,
}

impl WeatherWidget {
    pub fn new(cache: SharedTelemetryCache) -> Self {
        Self {
            state: WidgetState::Unloaded,
            cache,
            temp_signal: Signal::new("env.temperature", 22.5),
            humidity_signal: Signal::new("env.humidity", 55.0),
            material: MaterialSpec {
                material_type: MaterialType::Glass,
                tint_color: "#0F172A".to_string(),
                tint_opacity: 0.85,
                blur_radius: 20.0,
                ..Default::default()
            },
            budget: PerformanceBudget {
                target_cpu_pct: 0.02,
                target_memory_mb: 10.0,
                target_fps: 1,
                material_cost: "low".to_string(),
                animation_cost: "low".to_string(),
            },
            tick_count: 0,
        }
    }

    pub fn cache(&self) -> &SharedTelemetryCache {
        &self.cache
    }

    pub fn material(&self) -> &MaterialSpec {
        &self.material
    }

    pub fn budget(&self) -> &PerformanceBudget {
        &self.budget
    }
}

impl WidgetLifecycle for WeatherWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("[WeatherWidget] Plugin loaded & initialized.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("[WeatherWidget] Plugin mounted to desktop composition host.");
        Ok(())
    }

    fn on_update(&mut self, ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;
        let mut canvas = BatchRenderCanvas::new();

        // Background Glass Card
        canvas.draw_rect(
            RectF::new(0.0, 0.0, 320.0, 180.0),
            Color::from_u8(15, 23, 42, 215),
            12.0,
        );

        // Header Title
        canvas.draw_text(
            "Seattle, WA — Partly Cloudy",
            "Segoe UI",
            14.0,
            RectF::new(16.0, 16.0, 280.0, 24.0),
            Color::from_u8(148, 163, 184, 255),
        );

        // Main Temperature
        let temp_str = format!("{:.1}°C", self.temp_signal.value);
        canvas.draw_text(
            &temp_str,
            "Segoe UI Variable Display",
            36.0,
            RectF::new(16.0, 48.0, 160.0, 48.0),
            Color::from_u8(255, 255, 255, 255),
        );

        // Sub-metrics: Humidity & Wind
        let sub_str = format!("Humidity: {:.0}%  |  Wind: 12 km/h NW", self.humidity_signal.value);
        canvas.draw_text(
            &sub_str,
            "Segoe UI",
            12.0,
            RectF::new(16.0, 104.0, 280.0, 20.0),
            Color::from_u8(203, 213, 225, 255),
        );

        // 3-Day Mini Forecast
        canvas.draw_rect(
            RectF::new(16.0, 132.0, 84.0, 32.0),
            Color::from_u8(30, 41, 59, 180),
            6.0,
        );
        canvas.draw_text("Mon 23°", "Segoe UI", 11.0, RectF::new(24.0, 140.0, 68.0, 16.0), Color::from_u8(241, 245, 249, 255));

        canvas.draw_rect(
            RectF::new(108.0, 132.0, 84.0, 32.0),
            Color::from_u8(30, 41, 59, 180),
            6.0,
        );
        canvas.draw_text("Tue 21°", "Segoe UI", 11.0, RectF::new(116.0, 140.0, 68.0, 16.0), Color::from_u8(241, 245, 249, 255));

        canvas.draw_rect(
            RectF::new(200.0, 132.0, 84.0, 32.0),
            Color::from_u8(30, 41, 59, 180),
            6.0,
        );
        canvas.draw_text("Wed 25°", "Segoe UI", 11.0, RectF::new(208.0, 140.0, 68.0, 16.0), Color::from_u8(241, 245, 249, 255));

        if self.tick_count % 10 == 1 {
            info!("[WeatherWidget] Rendered frame #{frame} with {cmds} draw commands", frame = ctx.frame_index, cmds = canvas.commands().len());
        }

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("[WeatherWidget] Plugin unmounted.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("[WeatherWidget] Plugin unloaded.");
        Ok(())
    }

    fn state(&self) -> WidgetState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_widget_lifecycle() {
        let cache = SharedTelemetryCache::new();
        let mut widget = WeatherWidget::new(cache);
        assert_eq!(widget.state(), WidgetState::Unloaded);

        widget.on_load().unwrap();
        assert_eq!(widget.state(), WidgetState::Loaded);

        widget.on_mount().unwrap();
        assert_eq!(widget.state(), WidgetState::Mounted);

        let ctx = TickContext { timestamp_ms: 1000, delta_time_ms: 1000.0, frame_index: 1 };
        widget.on_update(&ctx).unwrap();

        widget.on_unmount().unwrap();
        assert_eq!(widget.state(), WidgetState::Unmounted);

        widget.on_unload().unwrap();
        assert_eq!(widget.state(), WidgetState::Unloaded);
    }

    #[test]
    fn test_weather_renderer_draw_commands() {
        let cache = SharedTelemetryCache::new();
        let mut widget = WeatherWidget::new(cache);
        widget.on_load().unwrap();
        widget.on_mount().unwrap();

        let ctx = TickContext { timestamp_ms: 1000, delta_time_ms: 1000.0, frame_index: 1 };
        assert!(widget.on_update(&ctx).is_ok());
    }

    #[test]
    fn test_weather_widget_empty_cache() {
        let cache = SharedTelemetryCache::new();
        let mut widget = WeatherWidget::new(cache);
        widget.on_load().unwrap();
        widget.on_mount().unwrap();
        let ctx = TickContext { timestamp_ms: 0, delta_time_ms: 1000.0, frame_index: 0 };
        assert!(widget.on_update(&ctx).is_ok());
    }
}
