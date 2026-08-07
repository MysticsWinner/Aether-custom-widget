use widget_sdk::{Color, DrawCommand, RectF};

/// Visual alignment grid and bounding box overlay generator.
#[derive(Debug, Clone)]
pub struct LayoutGridOverlay {
    enabled: bool,
    grid_spacing: f32,
    grid_color: Color,
}

impl LayoutGridOverlay {
    pub fn new(grid_spacing: f32) -> Self {
        Self {
            enabled: false,
            grid_spacing,
            grid_color: Color::rgba(0.0, 180.0, 255.0, 60.0),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generates bounding box draw commands for a target widget rectangle.
    pub fn generate_widget_bounds_overlay(&self, bounds: RectF, widget_id: &str) -> Vec<DrawCommand> {
        if !self.enabled {
            return Vec::new();
        }

        let mut commands = Vec::new();
        // Bounding outline
        commands.push(DrawCommand::DrawRect {
            rect: bounds,
            color: Color::rgba(255.0, 0.0, 128.0, 180.0),
            corner_radius: 2.0,
        });

        // Label
        let text_rect = RectF {
            x: bounds.x,
            y: (bounds.y - 20.0).max(0.0),
            width: bounds.width,
            height: 18.0,
        };

        commands.push(DrawCommand::DrawText {
            text: widget_id.to_string(),
            font_family: "Segoe UI".to_string(),
            font_size: 12.0,
            rect: text_rect,
            color: Color::rgba(255.0, 255.0, 255.0, 220.0),
        });

        commands
    }
}

impl Default for LayoutGridOverlay {
    fn default() -> Self {
        Self::new(20.0)
    }
}
