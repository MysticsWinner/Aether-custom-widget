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

/// Lifecycle phase of a touchscreen contact point (`WM_TOUCH` / `WM_POINTER`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TouchPhase {
    Down,
    Move,
    Up,
    Cancel,
}

/// Multi-touch contact event decoded from Win32 `WM_TOUCH` / `POINTER_TOUCH_INFO`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TouchEvent {
    pub pointer_id: u32,
    pub phase: TouchPhase,
    pub x: f32,
    pub y: f32,
    /// Normalized contact pressure in `[0.0, 1.0]`.
    pub pressure: f32,
    pub contact_width: f32,
    pub contact_height: f32,
}

impl TouchEvent {
    /// Decodes a Win32 `TOUCHINPUT` coordinate (expressed in 1/100ths of a pixel) and raw pressure (`0..=1024`).
    pub fn from_win32_touch_input(
        pointer_id: u32,
        phase: TouchPhase,
        x_centipixels: i32,
        y_centipixels: i32,
        raw_pressure: u32,
        width_centipixels: u32,
        height_centipixels: u32,
    ) -> Self {
        Self {
            pointer_id,
            phase,
            x: x_centipixels as f32 / 100.0,
            y: y_centipixels as f32 / 100.0,
            pressure: (raw_pressure as f32 / 1024.0).clamp(0.0, 1.0),
            contact_width: width_centipixels as f32 / 100.0,
            contact_height: height_centipixels as f32 / 100.0,
        }
    }

    /// Computes Euclidean distance between two active touch contacts for pinch-to-zoom gestures.
    pub fn pinch_distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Stylus / Digital Pen barrel button state (`POINTER_PEN_INFO`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenBarrelButton {
    None,
    Primary,
    Secondary,
}

/// Active stylus / pen input event decoded from Win32 `WM_POINTERUPDATE` (`POINTER_PEN_INFO`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PenEvent {
    pub pointer_id: u32,
    pub x: f32,
    pub y: f32,
    /// Normalized tip pressure in `[0.0, 1.0]` (mapped from Win32 `0..=1024`).
    pub pressure: f32,
    /// X-axis tilt in degrees `[-90.0, 90.0]`.
    pub tilt_x: f32,
    /// Y-axis tilt in degrees `[-90.0, 90.0]`.
    pub tilt_y: f32,
    /// Barrel rotation twist in degrees `[0.0, 359.0]`.
    pub twist_deg: f32,
    pub is_eraser: bool,
    pub is_inverted: bool,
    pub in_contact: bool,
    pub barrel_button: PenBarrelButton,
}

impl PenEvent {
    /// Decodes a Win32 `POINTER_PEN_INFO` structure into a normalized `PenEvent`.
    #[allow(clippy::too_many_arguments)]
    pub fn from_win32_pen_info(
        pointer_id: u32,
        x: f32,
        y: f32,
        raw_pressure_0_to_1024: u32,
        tilt_x_deg: i32,
        tilt_y_deg: i32,
        rotation_deg: u32,
        pen_flags: u32,
        in_contact: bool,
    ) -> Self {
        // Win32 PEN_FLAG constants: BARREL = 0x1, INVERTED = 0x2, ERASER = 0x4
        let has_barrel = (pen_flags & 0x1) != 0;
        let is_inverted = (pen_flags & 0x2) != 0;
        let is_eraser = (pen_flags & 0x4) != 0;
        Self {
            pointer_id,
            x,
            y,
            pressure: (raw_pressure_0_to_1024 as f32 / 1024.0).clamp(0.0, 1.0),
            tilt_x: (tilt_x_deg as f32).clamp(-90.0, 90.0),
            tilt_y: (tilt_y_deg as f32).clamp(-90.0, 90.0),
            twist_deg: (rotation_deg % 360) as f32,
            is_eraser,
            is_inverted,
            in_contact,
            barrel_button: if has_barrel {
                PenBarrelButton::Primary
            } else {
                PenBarrelButton::None
            },
        }
    }
}

/// Legacy pointer, touch, pen, and keyboard input events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    Click { x: f32, y: f32, button: u8 },
    Hover { x: f32, y: f32 },
    KeyDown { key_code: u32 },
    Pointer(PointerEvent),
    Drag(DragEvent),
    Scroll(ScrollEvent),
    Touch(TouchEvent),
    Pen(PenEvent),
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

    #[test]
    fn test_win32_touch_and_pen_event_decoding() {
        let t1 = TouchEvent::from_win32_touch_input(1, TouchPhase::Down, 10000, 20000, 512, 1200, 1200);
        let t2 = TouchEvent::from_win32_touch_input(2, TouchPhase::Move, 13000, 24000, 1024, 1500, 1500);
        assert_eq!(t1.x, 100.0);
        assert_eq!(t1.y, 200.0);
        assert!((t1.pressure - 0.5).abs() < 1e-4);
        assert_eq!(t1.contact_width, 12.0);
        assert!((t1.pinch_distance(&t2) - 50.0).abs() < 1e-4);

        let pen = PenEvent::from_win32_pen_info(7, 250.0, 180.0, 768, -15, 30, 45, 0x1 | 0x4, true);
        assert_eq!(pen.pointer_id, 7);
        assert!((pen.pressure - 0.75).abs() < 1e-4);
        assert_eq!(pen.tilt_x, -15.0);
        assert_eq!(pen.tilt_y, 30.0);
        assert_eq!(pen.twist_deg, 45.0);
        assert!(pen.is_eraser);
        assert!(!pen.is_inverted);
        assert_eq!(pen.barrel_button, PenBarrelButton::Primary);
    }
}
