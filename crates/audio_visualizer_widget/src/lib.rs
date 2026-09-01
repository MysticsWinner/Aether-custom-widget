//! Aether Spectrum Audio & Media Visualizer Widget
//!
//! A built-in high-performance Aether widget that captures real-time 16-band WASAPI audio FFT
//! spectrum bins, peak dB levels, and Windows Media SMTC playback data from `SharedTelemetryCache`,
//! rendering an interactive dark glassmorphism visualizer card with playback controls.

use anyhow::Result;
use system_providers::SharedTelemetryCache;
use theme_engine::{MaterialSpec, MaterialType};
use tracing::info;
use widget_sdk::events::{HitTarget, HitTestTree, PointerEvent};
use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
use widget_sdk::perf_budget::PerformanceBudget;
use widget_sdk::reactive::Signal;
use widget_sdk::render_config::RenderConfig;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas, RenderEffect};

/// Audio Visualizer Widget — implements the 6-pillar `WidgetLifecycle` SDK.
pub struct AudioVisualizerWidget {
    state: WidgetState,
    cache: SharedTelemetryCache,
    is_playing_signal: Signal<bool>,
    volume_signal: Signal<f32>,
    material: MaterialSpec,
    budget: PerformanceBudget,
    hit_tree: HitTestTree,
    tick_count: u64,
    config: RenderConfig,
}

impl AudioVisualizerWidget {
    /// Creates a new `AudioVisualizerWidget` bound to the shared telemetry cache.
    pub fn new(cache: SharedTelemetryCache) -> Self {
        let mut hit_tree = HitTestTree::new();
        // Register interactive playback control buttons
        hit_tree.register_target(HitTarget::new("btn_prev", RectF::new(20.0, 140.0, 36.0, 36.0)));
        hit_tree.register_target(HitTarget::new("btn_play_pause", RectF::new(66.0, 140.0, 36.0, 36.0)));
        hit_tree.register_target(HitTarget::new("btn_next", RectF::new(112.0, 140.0, 36.0, 36.0)));

        Self {
            state: WidgetState::Unloaded,
            cache,
            is_playing_signal: Signal::new("media.is_playing", false),
            volume_signal: Signal::new("media.volume", 75.0),
            material: MaterialSpec {
                material_type: MaterialType::Mica,
                tint_color: "#121218".to_string(),
                tint_opacity: 0.90,
                blur_radius: 35.0,
                ..Default::default()
            },
            budget: PerformanceBudget {
                target_cpu_pct: 0.08,
                target_memory_mb: 16.0,
                target_fps: 60,
                material_cost: "medium".to_string(),
                animation_cost: "low".to_string(),
            },
            hit_tree,
            tick_count: 0,
            config: RenderConfig::default(),
        }
    }

    /// Handles pointer interaction (e.g. clicking Play/Pause or Skip buttons).
    pub fn handle_pointer_event(&mut self, event: PointerEvent) -> Option<String> {
        if let PointerEvent::Down { x, y, .. } = event {
            if let Some(target) = self.hit_tree.hit_test(x, y) {
                let element_id = target.element_id.clone();
                match element_id.as_str() {
                    "btn_play_pause" => {
                        let cur = *self.is_playing_signal.get();
                        self.is_playing_signal.set(!cur);
                        info!("Audio visualizer toggled play/pause: {}", !cur);
                    }
                    "btn_next" => info!("Audio visualizer skip next track"),
                    "btn_prev" => info!("Audio visualizer previous track"),
                    _ => {}
                }
                return Some(element_id);
            }
        }
        None
    }

    /// Returns reference to internal hit-testing tree.
    pub fn hit_tree(&self) -> &HitTestTree {
        &self.hit_tree
    }
}

impl WidgetLifecycle for AudioVisualizerWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("AudioVisualizerWidget loaded.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("AudioVisualizerWidget mounted.");
        Ok(())
    }

    fn on_update(&mut self, _ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;
        let snap = self.cache.get_snapshot();
        let spectrum = snap.audio_spectrum;
        let media = snap.media_playback;

        self.is_playing_signal.set(media.is_playing);
        self.volume_signal.set(spectrum.master_volume_pct);

        let mut canvas = BatchRenderCanvas::new();

        // 1. Background Card with Blur Effect
        canvas.draw_effect(
            RenderEffect::GaussianBlur { radius: 15.0 },
            RectF::new(0.0, 0.0, 380.0, 200.0),
        );
        canvas.draw_rect(
            RectF::new(0.0, 0.0, 380.0, 200.0),
            Color::rgba(0.07, 0.07, 0.10, 0.92),
            16.0,
        );

        // 2. Track Title and Artist Header
        canvas.draw_text(
            &media.title,
            "Segoe UI",
            16.0,
            RectF::new(20.0, 16.0, 340.0, 24.0),
            Color::rgba(0.98, 0.98, 1.0, 1.0),
        );
        canvas.draw_text(
            &format!("{} • {}", media.artist, media.playback_status),
            "Segoe UI",
            12.0,
            RectF::new(20.0, 42.0, 340.0, 18.0),
            Color::rgba(0.0, 0.94, 1.0, 0.9), // Cyan accent
        );

        // 3. 16-Band Real-Time Spectrum Equalizer Bars
        let bar_width = 18.0;
        let bar_gap = 4.0;
        let start_x = 20.0;
        let max_bar_height = 55.0;
        let base_y = 125.0;

        for (i, &magnitude) in spectrum.fft_bins_16.iter().enumerate() {
            let bar_x = start_x + (i as f32 * (bar_width + bar_gap));
            let bar_height = (magnitude * max_bar_height).max(3.0);
            let bar_y = base_y - bar_height;

            // Gradient cyan to purple color
            let r = 0.0 + (i as f32 / 16.0) * 0.7;
            let g = 0.94 - (i as f32 / 16.0) * 0.4;
            let b = 1.0;

            canvas.draw_rect(
                RectF::new(bar_x, bar_y, bar_width, bar_height),
                Color::rgba(r, g, b, 0.85),
                3.0,
            );
        }

        // 4. Interactive Media Controls
        canvas.draw_rect(
            RectF::new(20.0, 145.0, 32.0, 32.0),
            Color::rgba(0.2, 0.2, 0.3, 0.6),
            8.0,
        );
        canvas.draw_text("⏮", "Segoe UI", 12.0, RectF::new(28.0, 152.0, 20.0, 20.0), Color::rgb(1.0, 1.0, 1.0));

        canvas.draw_rect(
            RectF::new(62.0, 145.0, 32.0, 32.0),
            Color::rgba(0.0, 0.94, 1.0, 0.8),
            8.0,
        );
        let play_symbol = if *self.is_playing_signal.get() { "⏸" } else { "▶" };
        canvas.draw_text(play_symbol, "Segoe UI", 14.0, RectF::new(72.0, 150.0, 20.0, 20.0), Color::rgb(0.0, 0.0, 0.0));

        canvas.draw_rect(
            RectF::new(104.0, 145.0, 32.0, 32.0),
            Color::rgba(0.2, 0.2, 0.3, 0.6),
            8.0,
        );
        canvas.draw_text("⏭", "Segoe UI", 12.0, RectF::new(112.0, 152.0, 20.0, 20.0), Color::rgb(1.0, 1.0, 1.0));

        // 5. Volume Meter Indicator
        canvas.draw_text(
            &format!("Vol: {:.0}%", *self.volume_signal.get()),
            "Segoe UI",
            12.0,
            RectF::new(290.0, 152.0, 70.0, 20.0),
            Color::rgba(0.8, 0.8, 0.9, 0.8),
        );

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("AudioVisualizerWidget unmounted.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("AudioVisualizerWidget unloaded.");
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
    use widget_sdk::events::MouseButton;

    #[test]
    fn test_audio_visualizer_lifecycle() {
        let cache = SharedTelemetryCache::new();
        let mut widget = AudioVisualizerWidget::new(cache);
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
    fn test_audio_visualizer_pointer_hit_test_interaction() {
        let cache = SharedTelemetryCache::new();
        let mut widget = AudioVisualizerWidget::new(cache);

        // Click play button at (70, 150)
        let clicked = widget.handle_pointer_event(PointerEvent::Down {
            x: 70.0,
            y: 150.0,
            button: MouseButton::Left,
        });
        assert_eq!(clicked, Some("btn_play_pause".to_string()));
        assert_eq!(*widget.is_playing_signal.get(), true);
    }

    #[test]
    fn test_audio_visualizer_renders_with_active_telemetry() {
        let cache = SharedTelemetryCache::new();
        let mut snap = TelemetrySnapshot::default();
        snap.audio_spectrum.is_active = true;
        snap.audio_spectrum.fft_bins_16 = [0.8; 16];
        snap.media_playback.title = "Synthwave Sunrise".to_string();
        snap.media_playback.artist = "Aether Wave".to_string();
        cache.update_snapshot(snap);

        let mut widget = AudioVisualizerWidget::new(cache);
        widget.on_load().unwrap();
        widget.on_mount().unwrap();

        let ctx = TickContext {
            tick_number: 1,
            delta_ms: 16.6,
            system_time_ms: 1000,
        };
        assert!(widget.on_update(&ctx).is_ok());
    }
}
