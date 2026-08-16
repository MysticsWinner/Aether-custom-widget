use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata describing a theme package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeMetadata {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
}

impl Default for ThemeMetadata {
    fn default() -> Self {
        Self {
            id: "theme.default.dark".to_string(),
            name: "Default Dark Theme".to_string(),
            author: "System".to_string(),
            version: "1.0.0".to_string(),
            description: "Default high-contrast dark theme".to_string(),
        }
    }
}

/// Font token definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FontConfig {
    pub family: String,
    pub size_pt: f32,
    pub weight: String,
    pub fallback: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "Segoe UI".to_string(),
            size_pt: 14.0,
            weight: "Normal".to_string(),
            fallback: "Arial".to_string(),
        }
    }
}

/// Layout spacing and backdrop rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutConfig {
    pub padding: f32,
    pub gap: f32,
    pub corner_radius: f32,
    pub backdrop: String, // "Mica", "Acrylic", "Transparent"
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            padding: 12.0,
            gap: 8.0,
            corner_radius: 8.0,
            backdrop: "Mica".to_string(),
        }
    }
}

/// Spring animation physics and easing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationConfig {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    pub easing: String,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            stiffness: 180.0,
            damping: 12.0,
            mass: 1.0,
            easing: "EaseOutQuad".to_string(),
        }
    }
}

use crate::tokens::DesignTokens;
use crate::material::MaterialSpec;

/// Comprehensive `theme.json` Schema supporting:
/// theme.json, tokens, materials, typography, motion, accessibility, theme inheritance (extends).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeSchema {
    pub metadata: ThemeMetadata,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub tokens: Option<DesignTokens>,
    #[serde(default)]
    pub materials: Option<HashMap<String, MaterialSpec>>,
    pub colors: HashMap<String, String>,
    pub fonts: HashMap<String, FontConfig>,
    pub icons: HashMap<String, String>,
    pub widgets: HashMap<String, HashMap<String, String>>,
    pub layouts: HashMap<String, LayoutConfig>,
    pub animations: HashMap<String, AnimationConfig>,
}

impl Default for ThemeSchema {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("theme.accent".into(), "#0078D7".into());
        colors.insert("theme.background".into(), "#1E1E1EE6".into());
        colors.insert("theme.text_primary".into(), "#FFFFFF".into());

        let mut fonts = HashMap::new();
        fonts.insert("default".into(), FontConfig::default());

        let mut icons = HashMap::new();
        icons.insert("cpu".into(), "assets/icons/cpu.svg".into());

        let mut widgets = HashMap::new();
        let mut sys_widget_override = HashMap::new();
        sys_widget_override.insert("card_color".into(), "#252526".into());
        widgets.insert("widget.sys_monitor.v1".into(), sys_widget_override);

        let mut layouts = HashMap::new();
        layouts.insert("default".into(), LayoutConfig::default());

        let mut animations = HashMap::new();
        animations.insert("default".into(), AnimationConfig::default());

        Self {
            metadata: ThemeMetadata::default(),
            extends: None,
            tokens: Some(DesignTokens::default()),
            materials: None,
            colors,
            fonts,
            icons,
            widgets,
            layouts,
            animations,
        }
    }
}

impl ThemeSchema {
    /// Parses a `theme.json` content string.
    pub fn parse_json(json_str: &str) -> anyhow::Result<Self> {
        let schema: ThemeSchema = serde_json::from_str(json_str)?;
        Ok(schema)
    }

    /// Serializes current `ThemeSchema` to JSON string.
    pub fn to_json(&self) -> anyhow::Result<String> {
        let json_str = serde_json::to_string_pretty(self)?;
        Ok(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_schema_json_roundtrip() {
        let default_theme = ThemeSchema::default();
        let json_str = default_theme.to_json().unwrap();
        assert!(json_str.contains("theme.accent"));
        assert!(json_str.contains("Segoe UI"));

        let parsed: ThemeSchema = ThemeSchema::parse_json(&json_str).unwrap();
        assert_eq!(parsed, default_theme);
    }
}
