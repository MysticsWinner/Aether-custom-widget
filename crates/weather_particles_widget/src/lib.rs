//! Ambient Weather & Atmospheric Particle Simulation Widget
//!
//! Renders live weather metrics (temperature, humidity, UV index, wind speed)
//! alongside real-time Direct2D physical particle simulations (rain splashes, drifting snow, atmospheric fog).

use anyhow::Result;
use core_engine::rendering::particles::{
    Particle, ParticleEmitter, ParticleFieldConfig, ParticlePhysicsEngine, ParticleType,
};
use system_providers::SharedTelemetryCache;
use theme_engine::{MaterialSpec, MaterialType};
use tracing::info;
use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
use widget_sdk::perf_budget::PerformanceBudget;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas, RenderEffect};

/// Weather condition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherState {
    ClearSun,
    RainStorm,
    SnowBlizzard,
    AtmosphericFog,
}

/// Ambient Weather & Physical Particle Simulation Widget.
pub struct WeatherParticlesWidget {
    state: WidgetState,
    _cache: SharedTelemetryCache,
    weather_state: WeatherState,
    temperature_c: f32,
    humidity_pct: f32,
    wind_speed_kmh: f32,
    uv_index: u32,
    emitter: ParticleEmitter,
    physics: ParticlePhysicsEngine,
    particles: Vec<Particle>,
    material: MaterialSpec,
    budget: PerformanceBudget,
    tick_count: u64,
}

impl WeatherParticlesWidget {
    /// Creates a new `WeatherParticlesWidget`.
    pub fn new(cache: SharedTelemetryCache) -> Self {
        let width = 420.0;
        let height = 240.0;
        let emitter = ParticleEmitter::new(ParticleType::RainDrop, 300, 120.0, width, height);
        let physics = ParticlePhysicsEngine::new(ParticleFieldConfig::default(), width, height);

        Self {
            state: WidgetState::Unloaded,
            _cache: cache,
            weather_state: WeatherState::RainStorm,
            temperature_c: 18.5,
            humidity_pct: 82.0,
            wind_speed_kmh: 24.0,
            uv_index: 3,
            emitter,
            physics,
            particles: Vec::with_capacity(300),
            material: MaterialSpec {
                material_type: MaterialType::Acrylic,
                tint_color: "#111827".to_string(),
                tint_opacity: 0.90,
                blur_radius: 35.0,
                ..Default::default()
            },
            budget: PerformanceBudget {
                target_cpu_pct: 0.05,
                target_memory_mb: 18.0,
                target_fps: 60,
                material_cost: "medium".to_string(),
                animation_cost: "medium".to_string(),
            },
            tick_count: 0,
        }
    }

    /// Sets the current atmospheric weather condition and switches particle emitter.
    pub fn set_weather_state(&mut self, state: WeatherState) {
        self.weather_state = state;
        self.particles.clear();
        let p_type = match state {
            WeatherState::ClearSun => ParticleType::SolarRay,
            WeatherState::RainStorm => ParticleType::RainDrop,
            WeatherState::SnowBlizzard => ParticleType::SnowFlake,
            WeatherState::AtmosphericFog => ParticleType::FogCloud,
        };
        self.emitter = ParticleEmitter::new(p_type, 300, 100.0, 420.0, 240.0);
    }

    /// Returns the live particle count.
    pub fn active_particles_count(&self) -> usize {
        self.particles.len()
    }
}

impl WidgetLifecycle for WeatherParticlesWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("WeatherParticlesWidget loaded.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("WeatherParticlesWidget mounted.");
        Ok(())
    }

    fn on_update(&mut self, ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;
        let dt = (ctx.delta_ms as f32 / 1000.0).clamp(0.001, 0.05);

        // 1. Emit and simulate physical particles
        self.emitter.emit(dt, &mut self.particles);

        let card_bounds = RectF::new(20.0, 160.0, 380.0, 60.0);
        self.physics.step(dt, &mut self.particles, &[card_bounds]);

        // 2. Render Canvas
        let mut canvas = BatchRenderCanvas::new();

        // Background Glass
        canvas.draw_effect(
            RenderEffect::GaussianBlur { radius: 25.0 },
            RectF::new(0.0, 0.0, 420.0, 240.0),
        );
        canvas.draw_rect(
            RectF::new(0.0, 0.0, 420.0, 240.0),
            Color::rgba(0.07, 0.09, 0.15, 0.90),
            18.0,
        );

        // Render Physical Particles
        for p in &self.particles {
            match p.p_type {
                ParticleType::RainDrop => {
                    canvas.draw_rect(
                        RectF::new(p.x, p.y, p.size, p.size * 5.0),
                        Color::rgba(0.4, 0.75, 1.0, p.opacity),
                        1.0,
                    );
                }
                ParticleType::SnowFlake => {
                    canvas.draw_rect(
                        RectF::new(p.x, p.y, p.size, p.size),
                        Color::rgba(0.95, 0.98, 1.0, p.opacity),
                        p.size / 2.0,
                    );
                }
                ParticleType::SplashDroplet => {
                    canvas.draw_rect(
                        RectF::new(p.x, p.y, p.size, p.size),
                        Color::rgba(0.6, 0.85, 1.0, p.opacity),
                        1.0,
                    );
                }
                _ => {
                    canvas.draw_rect(
                        RectF::new(p.x, p.y, p.size, p.size),
                        Color::rgba(1.0, 0.9, 0.5, p.opacity * 0.3),
                        p.size / 2.0,
                    );
                }
            }
        }

        // Weather Metrics Text
        let temp_str = format!("{:.1}°C", self.temperature_c);
        let condition_str = match self.weather_state {
            WeatherState::ClearSun => "Sunny & Clear",
            WeatherState::RainStorm => "Thunderstorm & Rain",
            WeatherState::SnowBlizzard => "Heavy Snow",
            WeatherState::AtmosphericFog => "Dense Mist",
        };

        canvas.draw_text(
            &temp_str,
            "Segoe UI Variable Display",
            34.0,
            RectF::new(25.0, 20.0, 160.0, 40.0),
            Color::rgb(0.95, 0.97, 1.0),
        );
        canvas.draw_text(
            condition_str,
            "Segoe UI Variable Text",
            14.0,
            RectF::new(25.0, 65.0, 200.0, 20.0),
            Color::rgb(0.0, 0.85, 1.0),
        );

        // Humidity & Wind Stats Bar
        let details_str = format!("Humidity: {:.0}%  •  Wind: {:.0} km/h  •  UV: {}", self.humidity_pct, self.wind_speed_kmh, self.uv_index);
        canvas.draw_text(
            &details_str,
            "Segoe UI Variable Text",
            12.0,
            RectF::new(25.0, 195.0, 370.0, 20.0),
            Color::rgba(0.7, 0.75, 0.85, 0.9),
        );

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("WeatherParticlesWidget unmounted.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("WeatherParticlesWidget unloaded.");
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
    fn test_weather_particles_widget_lifecycle() {
        let cache = SharedTelemetryCache::new();
        let mut widget = WeatherParticlesWidget::new(cache);
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
        assert!(widget.active_particles_count() > 0, "Particles should be active after tick");

        assert!(widget.on_unmount().is_ok());
        assert_eq!(widget.state(), WidgetState::Unmounted);

        assert!(widget.on_unload().is_ok());
        assert_eq!(widget.state(), WidgetState::Unloaded);
    }

    #[test]
    fn test_weather_condition_switching() {
        let cache = SharedTelemetryCache::new();
        let mut widget = WeatherParticlesWidget::new(cache);
        assert_eq!(widget.weather_state, WeatherState::RainStorm);

        widget.set_weather_state(WeatherState::SnowBlizzard);
        assert_eq!(widget.weather_state, WeatherState::SnowBlizzard);
        assert_eq!(widget.emitter.particle_type(), ParticleType::SnowFlake);

        widget.set_weather_state(WeatherState::ClearSun);
        assert_eq!(widget.weather_state, WeatherState::ClearSun);
        assert_eq!(widget.emitter.particle_type(), ParticleType::SolarRay);
    }
}
