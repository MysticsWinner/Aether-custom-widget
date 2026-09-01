//! Interactive Hit-Testing, Gesture & Event Dispatching Pipeline
//!
//! Maps cursor and touch input coordinates to hierarchical element bounding boxes,
//! delivering discrete pointer, hover, drag, and scroll events to widgets.

use crate::rendering::RectF;
use serde::{Deserialize, Serialize};

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Discrete pointer input events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PointerEvent {
    Down { x: f32, y: f32, button: MouseButton },
    Up { x: f32, y: f32, button: MouseButton },
    Move { x: f32, y: f32 },
    Enter { x: f32, y: f32 },
    Leave,
}

/// Continuous drag and inertial gesture events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DragEvent {
    Start { start_x: f32, start_y: f32 },
    Update { delta_x: f32, delta_y: f32, total_x: f32, total_y: f32 },
    End { velocity_x: f32, velocity_y: f32 },
}

/// Scroll wheel input events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollEvent {
    pub x: f32,
    pub y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Legacy pointer and keyboard input events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    Click { x: f32, y: f32, button: u8 },
    Hover { x: f32, y: f32 },
    KeyDown { key_code: u32 },
    Pointer(PointerEvent),
    Drag(DragEvent),
    Scroll(ScrollEvent),
}

/// Unified widget event payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidgetEvent {
    Input(InputEvent),
    Telemetry { metric: String, value: f64 },
    ThemeChanged { theme_name: String },
    Custom { topic: String, payload: String },
}

/// Interactive element hit-test target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HitTarget {
    pub element_id: String,
    pub bounds: RectF,
    pub z_index: i32,
    pub is_draggable: bool,
    pub is_clickable: bool,
}

impl HitTarget {
    pub fn new(element_id: impl Into<String>, bounds: RectF) -> Self {
        Self {
            element_id: element_id.into(),
            bounds,
            z_index: 0,
            is_draggable: false,
            is_clickable: true,
        }
    }

    /// Evaluates if coordinate (x, y) falls within element bounds.
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.bounds.x
            && x <= (self.bounds.x + self.bounds.width)
            && y >= self.bounds.y
            && y <= (self.bounds.y + self.bounds.height)
    }
}

/// Hierarchical Hit-Testing Dispatcher Tree.
#[derive(Debug, Clone, Default)]
pub struct HitTestTree {
    targets: Vec<HitTarget>,
}

impl HitTestTree {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    /// Registers a hit-testable element target.
    pub fn register_target(&mut self, target: HitTarget) {
        self.targets.push(target);
    }

    /// Clears all targets before a new frame layout pass.
    pub fn clear(&mut self) {
        self.targets.clear();
    }

    /// Evaluates cursor position (x, y) and returns topmost matching element.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<&HitTarget> {
        let mut matches: Vec<&HitTarget> = self
            .targets
            .iter()
            .filter(|t| t.contains_point(x, y))
            .collect();

        // Sort by z_index descending (topmost element wins)
        matches.sort_by(|a, b| b.z_index.cmp(&a.z_index));
        matches.first().copied()
    }
}

/// 4. Events API Pillar Interface
pub trait EventSubscriber: Send + Sync {
    fn on_event(&mut self, event: &WidgetEvent) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEventSubscriber {
        received_count: usize,
    }

    impl EventSubscriber for MockEventSubscriber {
        fn on_event(&mut self, _event: &WidgetEvent) -> anyhow::Result<()> {
            self.received_count += 1;
            Ok(())
        }
    }

    #[test]
    fn test_event_subscriber() {
        let mut subscriber = MockEventSubscriber { received_count: 0 };
        let event = WidgetEvent::Telemetry {
            metric: "sys.cpu".to_string(),
            value: 52.4,
        };
        subscriber.on_event(&event).unwrap();
        assert_eq!(subscriber.received_count, 1);
    }

    #[test]
    fn test_hit_test_tree_evaluation() {
        let mut tree = HitTestTree::new();
        tree.register_target(HitTarget {
            element_id: "card_background".to_string(),
            bounds: RectF::new(0.0, 0.0, 300.0, 200.0),
            z_index: 0,
            is_draggable: true,
            is_clickable: false,
        });

        tree.register_target(HitTarget {
            element_id: "button_refresh".to_string(),
            bounds: RectF::new(200.0, 20.0, 80.0, 30.0),
            z_index: 10,
            is_draggable: false,
            is_clickable: true,
        });

        // Hit button
        let hit1 = tree.hit_test(210.0, 25.0).expect("Should hit button");
        assert_eq!(hit1.element_id, "button_refresh");

        // Hit background
        let hit2 = tree.hit_test(50.0, 50.0).expect("Should hit background");
        assert_eq!(hit2.element_id, "card_background");

        // Miss all
        let miss = tree.hit_test(400.0, 400.0);
        assert!(miss.is_none());
    }
}
