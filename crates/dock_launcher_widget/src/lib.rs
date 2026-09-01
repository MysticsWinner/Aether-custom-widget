//! Aether Dynamic Desktop Dock & App Launcher Widget
//!
//! A modern, hardware-accelerated desktop application dock featuring spring magnification physics,
//! vector iconography, active process indicator dots, and interactive hit testing.

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

/// Dock item representation.
#[derive(Debug, Clone)]
pub struct DockItem {
    pub id: String,
    pub label: String,
    pub icon_path: String,
    pub is_running: bool,
    pub current_scale: f32,
    pub target_scale: f32,
}

/// Dynamic Desktop Dock Widget — implements the 6-pillar `WidgetLifecycle` SDK.
pub struct DockLauncherWidget {
    state: WidgetState,
    _cache: SharedTelemetryCache,
    items: Vec<DockItem>,
    hovered_index: Option<usize>,
    material: MaterialSpec,
    budget: PerformanceBudget,
    hit_tree: HitTestTree,
    item_clicked_signal: Signal<String>,
    tick_count: u64,
    config: RenderConfig,
}

impl DockLauncherWidget {
    /// Creates a new `DockLauncherWidget`.
    pub fn new(cache: SharedTelemetryCache) -> Self {
        let items = vec![
            DockItem {
                id: "dock_browser".to_string(),
                label: "Browser".to_string(),
                icon_path: "M 10 0 C 15 0 20 5 20 10 C 20 15 15 20 10 20 C 5 20 0 15 0 10 C 0 5 5 0 10 0 Z".to_string(),
                is_running: true,
                current_scale: 1.0,
                target_scale: 1.0,
            },
            DockItem {
                id: "dock_editor".to_string(),
                label: "Code".to_string(),
                icon_path: "M 5 0 L 15 0 L 20 20 L 0 20 Z".to_string(),
                is_running: true,
                current_scale: 1.0,
                target_scale: 1.0,
            },
            DockItem {
                id: "dock_terminal".to_string(),
                label: "Terminal".to_string(),
                icon_path: "M 0 0 L 20 0 L 20 15 L 0 15 Z".to_string(),
                is_running: false,
                current_scale: 1.0,
                target_scale: 1.0,
            },
            DockItem {
                id: "dock_music".to_string(),
                label: "Music".to_string(),
                icon_path: "M 5 0 L 15 0 L 15 15 L 5 15 Z".to_string(),
                is_running: true,
                current_scale: 1.0,
                target_scale: 1.0,
            },
            DockItem {
                id: "dock_settings".to_string(),
                label: "Settings".to_string(),
                icon_path: "M 2 2 L 18 2 L 18 18 L 2 18 Z".to_string(),
                is_running: false,
                current_scale: 1.0,
                target_scale: 1.0,
            },
        ];

        let mut hit_tree = HitTestTree::new();
        let base_x = 25.0;
        let item_spacing = 90.0;
        for (i, item) in items.iter().enumerate() {
            let x = base_x + (i as f32 * item_spacing);
            hit_tree.register_target(HitTarget::new(&item.id, RectF::new(x, 15.0, 55.0, 55.0)));
        }

        Self {
            state: WidgetState::Unloaded,
            _cache: cache,
            items,
            hovered_index: None,
            material: MaterialSpec {
                material_type: MaterialType::Mica,
                tint_color: "#161B22".to_string(),
                tint_opacity: 0.88,
                blur_radius: 40.0,
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
            item_clicked_signal: Signal::new("dock.clicked", "".to_string()),
            tick_count: 0,
            config: RenderConfig::default(),
        }
    }

    /// Handles pointer interaction (hover magnification & clicks).
    pub fn handle_pointer_event(&mut self, event: PointerEvent) -> Option<String> {
        match event {
            PointerEvent::Move { x, y } => {
                let mut found_hover = None;
                for (i, item) in self.items.iter().enumerate() {
                    let base_x = 25.0 + (i as f32 * 90.0);
                    if x >= base_x && x <= base_x + 60.0 && y >= 10.0 && y <= 70.0 {
                        found_hover = Some(i);
                        break;
                    }
                }
                self.hovered_index = found_hover;

                // Update target scales with gaussian magnification spread
                for (i, item) in self.items.iter_mut().enumerate() {
                    if let Some(h_idx) = self.hovered_index {
                        let dist = (i as f32 - h_idx as f32).abs();
                        if dist == 0.0 {
                            item.target_scale = 1.35;
                        } else if dist == 1.0 {
                            item.target_scale = 1.15;
                        } else {
                            item.target_scale = 1.0;
                        }
                    } else {
                        item.target_scale = 1.0;
                    }
                }
                None
            }
            PointerEvent::Down { x, y, .. } => {
                if let Some(target) = self.hit_tree.hit_test(x, y) {
                    let id = target.element_id.clone();
                    info!("Dock item launched: {}", id);
                    self.item_clicked_signal.set(id.clone());
                    Some(id)
                } else {
                    None
                }
            }
            PointerEvent::Leave => {
                self.hovered_index = None;
                for item in self.items.iter_mut() {
                    item.target_scale = 1.0;
                }
                None
            }
            _ => None,
        }
    }

    /// Returns reference to dock items.
    pub fn items(&self) -> &[DockItem] {
        &self.items
    }
}

impl WidgetLifecycle for DockLauncherWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("DockLauncherWidget loaded.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("DockLauncherWidget mounted.");
        Ok(())
    }

    fn on_update(&mut self, _ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;

        // Animate spring smoothing towards target scale
        for item in self.items.iter_mut() {
            let diff = item.target_scale - item.current_scale;
            item.current_scale += diff * 0.25; // Spring easing
        }

        let mut canvas = BatchRenderCanvas::new();

        // 1. Background Glass Capsule
        canvas.draw_effect(
            RenderEffect::GaussianBlur { radius: 20.0 },
            RectF::new(0.0, 0.0, 500.0, 80.0),
        );
        canvas.draw_rect(
            RectF::new(0.0, 0.0, 500.0, 80.0),
            Color::rgba(0.09, 0.11, 0.15, 0.90),
            24.0,
        );

        // 2. Render Dock Icons with Magnification Scales
        let base_x = 25.0;
        let item_spacing = 90.0;

        for (i, item) in self.items.iter().enumerate() {
            let cx = base_x + (i as f32 * item_spacing) + 25.0;
            let cy = 40.0;
            let size = 48.0 * item.current_scale;
            let half = size / 2.0;

            // Icon Background Tile
            canvas.draw_rect(
                RectF::new(cx - half, cy - half, size, size),
                Color::rgba(0.18, 0.22, 0.30, 0.85),
                12.0 * item.current_scale,
            );

            // Vector Icon
            canvas.draw_path(
                &item.icon_path,
                Some(Color::rgba(0.0, 0.88, 1.0, 0.95)),
                None,
                1.5,
            );

            // Active process indicator dot below icon
            if item.is_running {
                canvas.draw_rect(
                    RectF::new(cx - 2.5, 68.0, 5.0, 5.0),
                    Color::rgb(0.0, 0.95, 0.85),
                    2.5,
                );
            }
        }

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("DockLauncherWidget unmounted.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("DockLauncherWidget unloaded.");
        Ok(())
    }

    fn state(&self) -> WidgetState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use widget_sdk::events::MouseButton;

    #[test]
    fn test_dock_launcher_lifecycle() {
        let cache = SharedTelemetryCache::new();
        let mut widget = DockLauncherWidget::new(cache);
        assert_eq!(widget.state(), WidgetState::Unloaded);

        assert!(widget.on_load().is_ok());
        assert_eq!(widget.state(), WidgetState::Loaded);

        assert!(widget.on_mount().is_ok());
        assert_eq!(widget.state(), WidgetState::Mounted);

        let ctx = TickContext {
            timestamp_ms: 1000,
            delta_time_ms: 16.0,
            frame_index: 1,
        };
        assert!(widget.on_update(&ctx).is_ok());

        assert!(widget.on_unmount().is_ok());
        assert_eq!(widget.state(), WidgetState::Unmounted);

        assert!(widget.on_unload().is_ok());
        assert_eq!(widget.state(), WidgetState::Unloaded);
    }

    #[test]
    fn test_dock_launcher_pointer_hover_magnification() {
        let cache = SharedTelemetryCache::new();
        let mut widget = DockLauncherWidget::new(cache);

        // Hover over center item (item index 2)
        widget.handle_pointer_event(PointerEvent::Move { x: 230.0, y: 35.0 });
        assert_eq!(widget.hovered_index, Some(2));
        assert_eq!(widget.items[2].target_scale, 1.35);
        assert_eq!(widget.items[1].target_scale, 1.15);
        assert_eq!(widget.items[3].target_scale, 1.15);

        // Leave hover
        widget.handle_pointer_event(PointerEvent::Leave);
        assert_eq!(widget.hovered_index, None);
        assert_eq!(widget.items[2].target_scale, 1.0);
    }

    #[test]
    fn test_dock_launcher_click_launches_item() {
        let cache = SharedTelemetryCache::new();
        let mut widget = DockLauncherWidget::new(cache);

        let clicked = widget.handle_pointer_event(PointerEvent::Down {
            x: 40.0,
            y: 30.0,
            button: MouseButton::Left,
        });
        assert_eq!(clicked, Some("dock_browser".to_string()));
    }
}
