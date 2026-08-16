//! Aether AI Workstation Assistant & Desktop Profile Widget Plugin
//!
//! Emits Direct2D draw commands displaying active Desktop Profile status (`Gaming`, `Coding`, `Work`),
//! context signal triggers, and AI optimization recommendations.

use anyhow::Result;
use config_manager::{DesktopProfile, ProfileType};
use system_providers::SharedTelemetryCache;
use theme_engine::{MaterialSpec, MaterialType};
use tracing::info;
use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
use widget_sdk::perf_budget::PerformanceBudget;
use widget_sdk::reactive::Signal;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas};

pub struct AiAssistantWidget {
    state: WidgetState,
    cache: SharedTelemetryCache,
    active_profile: DesktopProfile,
    advice_signal: Signal<String>,
    material: MaterialSpec,
    budget: PerformanceBudget,
    tick_count: u64,
}

impl AiAssistantWidget {
    pub fn new(cache: SharedTelemetryCache) -> Self {
        Self {
            state: WidgetState::Unloaded,
            cache,
            active_profile: DesktopProfile {
                id: "profile.coding".to_string(),
                name: "Coding Workstation".to_string(),
                profile_type: ProfileType::Coding,
                target_fps: 60,
                enable_materials: true,
                ..Default::default()
            },
            advice_signal: Signal::new(
                "ai.advice",
                "VS Code active — Auto-switched to Coding Profile (materials enabled, 60 FPS budget)".to_string(),
            ),
            material: MaterialSpec {
                material_type: MaterialType::Mica,
                tint_color: "#181825".to_string(),
                tint_opacity: 0.9,
                blur_radius: 30.0,
                ..Default::default()
            },
            budget: PerformanceBudget {
                target_cpu_pct: 0.03,
                target_memory_mb: 15.0,
                target_fps: 5,
                material_cost: "low".to_string(),
                animation_cost: "low".to_string(),
            },
            tick_count: 0,
        }
    }

    pub fn set_active_profile(&mut self, profile: DesktopProfile) {
        self.active_profile = profile;
    }
}

impl WidgetLifecycle for AiAssistantWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("[AiAssistantWidget] Plugin loaded.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("[AiAssistantWidget] Plugin mounted.");
        Ok(())
    }

    fn on_update(&mut self, ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;        let mut canvas = BatchRenderCanvas::new();

        // Main Container
        canvas.draw_rect(
            RectF::new(0.0, 0.0, 340.0, 190.0),
            Color::from_u8(24, 24, 37, 230),
            12.0,
        );

        // Header Title & AI Badge
        canvas.draw_text(
            "AETHER AI COMPOSER & ASSISTANT",
            "Segoe UI",
            12.0,
            RectF::new(16.0, 16.0, 240.0, 20.0),
            Color::from_u8(192, 132, 252, 255), // Purple AI Accent
        );

        canvas.draw_rect(RectF::new(260.0, 14.0, 64.0, 22.0), Color::from_u8(147, 51, 234, 255), 4.0);
        canvas.draw_text("ACTIVE", "Segoe UI", 10.0, RectF::new(272.0, 18.0, 48.0, 14.0), Color::from_u8(255, 255, 255, 255));

        // Active Profile Box
        canvas.draw_rect(
            RectF::new(16.0, 44.0, 308.0, 46.0),
            Color::from_u8(39, 39, 58, 200),
            8.0,
        );
        let profile_title = format!("Profile: {}", self.active_profile.name);
        canvas.draw_text(
            &profile_title,
            "Segoe UI Variable Display",
            16.0,
            RectF::new(28.0, 52.0, 280.0, 22.0),
            Color::from_u8(255, 255, 255, 255),
        );
        let fps_info = format!("Target: {} FPS  |  Materials: {}", self.active_profile.target_fps, if self.active_profile.enable_materials { "ON" } else { "OFF" });
        canvas.draw_text(
            &fps_info,
            "Segoe UI",
            11.0,
            RectF::new(28.0, 72.0, 280.0, 16.0),
            Color::from_u8(148, 163, 184, 255),
        );

        // Context Advice Text
        canvas.draw_text(
            "AI Context Automation:",
            "Segoe UI",
            11.0,
            RectF::new(16.0, 100.0, 308.0, 16.0),
            Color::from_u8(148, 163, 184, 255),
        );
        canvas.draw_text(
            &self.advice_signal.value,
            "Segoe UI",
            12.0,
            RectF::new(16.0, 120.0, 308.0, 40.0),
            Color::from_u8(226, 232, 240, 255),
        );

        // Quick Profile Toggle Buttons
        canvas.draw_rect(RectF::new(16.0, 156.0, 94.0, 24.0), Color::from_u8(59, 130, 246, 200), 4.0);
        canvas.draw_text("Coding", "Segoe UI", 11.0, RectF::new(42.0, 160.0, 50.0, 16.0), Color::from_u8(255, 255, 255, 255));

        canvas.draw_rect(RectF::new(123.0, 156.0, 94.0, 24.0), Color::from_u8(34, 197, 94, 200), 4.0);
        canvas.draw_text("Gaming", "Segoe UI", 11.0, RectF::new(148.0, 160.0, 50.0, 16.0), Color::from_u8(255, 255, 255, 255));

        canvas.draw_rect(RectF::new(230.0, 156.0, 94.0, 24.0), Color::from_u8(168, 85, 247, 200), 4.0);
        canvas.draw_text("Minimal", "Segoe UI", 11.0, RectF::new(252.0, 160.0, 50.0, 16.0), Color::from_u8(255, 255, 255, 255));

        if self.tick_count % 10 == 1 {
            info!("[AiAssistantWidget] Frame #{frame} | Cmds: {cmds}", frame = ctx.frame_index, cmds = canvas.commands().len());
        }

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("[AiAssistantWidget] Plugin unmounted.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("[AiAssistantWidget] Plugin unloaded.");
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
    fn test_ai_widget_lifecycle() {
        let cache = SharedTelemetryCache::new();
        let mut widget = AiAssistantWidget::new(cache);
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
    fn test_ai_renderer_draw_commands() {
        let cache = SharedTelemetryCache::new();
        let mut widget = AiAssistantWidget::new(cache);
        widget.on_load().unwrap();
        widget.on_mount().unwrap();

        let ctx = TickContext { timestamp_ms: 1000, delta_time_ms: 1000.0, frame_index: 1 };
        assert!(widget.on_update(&ctx).is_ok());
    }

    #[test]
    fn test_ai_widget_profile_switch() {
        let cache = SharedTelemetryCache::new();
        let mut widget = AiAssistantWidget::new(cache);
        let gaming_profile = DesktopProfile {
            id: "profile.gaming".to_string(),
            name: "Gaming Mode".to_string(),
            profile_type: ProfileType::Gaming,
            target_fps: 120,
            enable_materials: false,
            ..Default::default()
        };
        widget.set_active_profile(gaming_profile.clone());
        assert_eq!(widget.active_profile.id, "profile.gaming");
        assert_eq!(widget.active_profile.target_fps, 120);
    }
}
