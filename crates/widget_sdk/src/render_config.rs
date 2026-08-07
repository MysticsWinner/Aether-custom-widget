use crate::display_target::{DesktopLayer, DisplayTarget};
use crate::rendering::Color;
use serde::{Deserialize, Serialize};

/// Custom widget rendering & display configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderConfig {
    /// Background opacity (range: 0.0 to 1.0).
    pub opacity: f32,
    /// Glass blur style preset ("Acrylic", "Mica", "DarkGlass", "None").
    pub blur_style: String,
    /// Optional custom accent color override.
    pub custom_accent: Option<Color>,
    /// Optional custom background color override.
    pub custom_bg: Option<Color>,
    /// Target monitor selection.
    pub display_target: DisplayTarget,
    /// Window Z-order layer pinning.
    pub desktop_layer: DesktopLayer,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            opacity: 0.92,
            blur_style: "DarkGlass".to_string(),
            custom_accent: None,
            custom_bg: None,
            display_target: DisplayTarget::PrimaryMonitor,
            desktop_layer: DesktopLayer::DesktopOverlay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_config_default_values() {
        let config = RenderConfig::default();
        assert_eq!(config.opacity, 0.92);
        assert_eq!(config.blur_style, "DarkGlass");
        assert_eq!(config.display_target, DisplayTarget::PrimaryMonitor);
        assert_eq!(config.desktop_layer, DesktopLayer::DesktopOverlay);
    }
}
