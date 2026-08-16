//! Aether Design Token System
//!
//! Provides first-class semantic design tokens categorized into 12 core domains:
//! Colors, Typography, Spacing, Sizing, Shape, Borders, Elevation,
//! Materials, Motion, Opacity, Accessibility, Performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic Design Tokens covering all 12 core visual and platform categories.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesignTokens {
    pub colors: HashMap<String, String>,
    pub typography: HashMap<String, String>,
    pub spacing: HashMap<String, f32>,
    pub sizing: HashMap<String, f32>,
    pub shape: HashMap<String, f32>,
    pub borders: HashMap<String, String>,
    pub elevation: HashMap<String, String>,
    pub materials: HashMap<String, String>,
    pub motion: HashMap<String, String>,
    pub opacity: HashMap<String, f32>,
    pub accessibility: HashMap<String, String>,
    pub performance: HashMap<String, String>,
}

impl Default for DesignTokens {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("accent".to_string(), "#0078D7".to_string());
        colors.insert("background".to_string(), "#1E1E1EE6".to_string());
        colors.insert("surface".to_string(), "#252526".to_string());
        colors.insert("border".to_string(), "#3E3E42".to_string());
        colors.insert("text_primary".to_string(), "#FFFFFF".to_string());
        colors.insert("text_secondary".to_string(), "#CCCCCC".to_string());

        let mut typography = HashMap::new();
        typography.insert("family_ui".to_string(), "Segoe UI".to_string());
        typography.insert("family_mono".to_string(), "Consolas".to_string());

        let mut spacing = HashMap::new();
        spacing.insert("xs".to_string(), 4.0);
        spacing.insert("sm".to_string(), 8.0);
        spacing.insert("md".to_string(), 12.0);
        spacing.insert("lg".to_string(), 16.0);
        spacing.insert("xl".to_string(), 24.0);

        let mut sizing = HashMap::new();
        sizing.insert("icon_sm".to_string(), 16.0);
        sizing.insert("icon_md".to_string(), 24.0);
        sizing.insert("card_min_width".to_string(), 200.0);

        let mut shape = HashMap::new();
        shape.insert("corner_radius_sm".to_string(), 4.0);
        shape.insert("corner_radius_md".to_string(), 8.0);
        shape.insert("corner_radius_lg".to_string(), 12.0);

        let mut borders = HashMap::new();
        borders.insert("thin".to_string(), "1px solid #3E3E42".to_string());
        borders.insert("focus".to_string(), "2px solid #0078D7".to_string());

        let mut elevation = HashMap::new();
        elevation.insert("level_0".to_string(), "none".to_string());
        elevation.insert("level_1".to_string(), "0 2px 4px rgba(0,0,0,0.2)".to_string());

        let mut materials = HashMap::new();
        materials.insert("card_surface".to_string(), "Mica".to_string());
        materials.insert("overlay_surface".to_string(), "Acrylic".to_string());

        let mut motion = HashMap::new();
        motion.insert("duration_fast".to_string(), "150ms".to_string());
        motion.insert("duration_normal".to_string(), "300ms".to_string());

        let mut opacity = HashMap::new();
        opacity.insert("disabled".to_string(), 0.38);
        opacity.insert("secondary".to_string(), 0.60);
        opacity.insert("full".to_string(), 1.0);

        let mut accessibility = HashMap::new();
        accessibility.insert("high_contrast".to_string(), "false".to_string());
        accessibility.insert("reduce_motion".to_string(), "false".to_string());

        let mut performance = HashMap::new();
        performance.insert("tier".to_string(), "balanced".to_string());

        Self {
            colors,
            typography,
            spacing,
            sizing,
            shape,
            borders,
            elevation,
            materials,
            motion,
            opacity,
            accessibility,
            performance,
        }
    }
}

impl DesignTokens {
    /// Resolves variable references in token values e.g. `{colors.accent}`
    pub fn resolve_value(&self, category: &str, key: &str) -> Option<String> {
        match category {
            "colors" => self.colors.get(key).cloned(),
            "typography" => self.typography.get(key).cloned(),
            "borders" => self.borders.get(key).cloned(),
            "elevation" => self.elevation.get(key).cloned(),
            "materials" => self.materials.get(key).cloned(),
            "motion" => self.motion.get(key).cloned(),
            "accessibility" => self.accessibility.get(key).cloned(),
            "performance" => self.performance.get(key).cloned(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_tokens_default_coverage() {
        let tokens = DesignTokens::default();
        assert_eq!(tokens.colors.get("accent").unwrap(), "#0078D7");
        assert_eq!(tokens.spacing.get("md").copied(), Some(12.0));
        assert_eq!(tokens.shape.get("corner_radius_md").copied(), Some(8.0));
        assert_eq!(tokens.resolve_value("materials", "card_surface").unwrap(), "Mica");
    }
}
