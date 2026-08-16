// widget_sdk/src/config.rs
// Copyright (c) Aether Platform. Licensed under the MIT License.
//
// Per-widget runtime configuration persisted as JSON under
// %LOCALAPPDATA%\Aether\widget_settings\<widget_id>.json

use serde::{Deserialize, Serialize};

/// Swap mode for the QuickSwap action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SwapMode {
    /// Swap the (x, y) desktop positions of the two widgets.
    Position,
    /// Swap the full WidgetConfig (display options, colours, quick_actions) of the two widgets.
    Configuration,
}

/// Optional per-channel colour overrides.  Stored as ARGB u32 (0xAARRGGBB).
/// When `None` the engine picks high-contrast tokens automatically from `theme_engine`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColourOverrides {
    /// Foreground (text/icon) colour override (ARGB u32).
    pub foreground: Option<u32>,
    /// Background colour override (ARGB u32).
    pub background: Option<u32>,
}

/// Rendering display options for a widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
    /// Opacity value clamped to [0.0, 1.0].
    pub opacity: f32,
    /// Scale factor (1.0 = 100%).
    pub scale: f32,
    /// Whether the widget position is drag-locked.
    pub locked: bool,
    /// Whether the widget is enabled (visible and updating).
    pub enabled: bool,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            scale: 1.0,
            locked: false,
            enabled: true,
        }
    }
}

/// Full configuration for a single widget, stored per-widget in JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// Widget ID (matches manifest `metadata.id`).
    pub widget_id: String,
    /// Rendering / display options.
    pub display_options: DisplayOptions,
    /// Optional explicit colour overrides; `None` means auto high-contrast.
    pub colour_overrides: Option<ColourOverrides>,
    /// Quick-swap enabled flag.
    pub quick_swap: bool,
}

impl WidgetConfig {
    /// Construct a default config for a given widget ID.
    pub fn new(widget_id: &str) -> Self {
        Self {
            widget_id: widget_id.to_string(),
            display_options: DisplayOptions::default(),
            colour_overrides: None,
            quick_swap: false,
        }
    }
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self::new("unknown")
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_config_default_values() {
        let cfg = WidgetConfig::new("perf_monitor");
        assert_eq!(cfg.widget_id, "perf_monitor");
        assert!((cfg.display_options.opacity - 1.0).abs() < f32::EPSILON);
        assert!((cfg.display_options.scale - 1.0).abs() < f32::EPSILON);
        assert!(!cfg.display_options.locked);
        assert!(cfg.display_options.enabled);
        assert!(!cfg.quick_swap);
        assert!(cfg.colour_overrides.is_none());
    }

    #[test]
    fn test_widget_config_serialization_roundtrip() {
        let cfg = WidgetConfig {
            widget_id: "weather_widget".to_string(),
            display_options: DisplayOptions {
                opacity: 0.8,
                scale: 1.2,
                locked: true,
                enabled: true,
            },
            colour_overrides: Some(ColourOverrides {
                foreground: Some(0xFFFFFFFF),
                background: Some(0xCC000000),
            }),
            quick_swap: true,
        };

        let json = serde_json::to_string(&cfg).expect("serialize");
        let decoded: WidgetConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.widget_id, cfg.widget_id);
        assert!((decoded.display_options.opacity - 0.8).abs() < f32::EPSILON);
        assert!(decoded.display_options.locked);
        assert!(decoded.quick_swap);
        assert!(decoded.colour_overrides.is_some());
    }

    #[test]
    fn test_swap_mode_serialization() {
        let pos = SwapMode::Position;
        let cfg = SwapMode::Configuration;
        let pos_json = serde_json::to_string(&pos).unwrap();
        let cfg_json = serde_json::to_string(&cfg).unwrap();
        assert_eq!(pos_json, "\"position\"");
        assert_eq!(cfg_json, "\"configuration\"");
        let decoded_pos: SwapMode = serde_json::from_str(&pos_json).unwrap();
        assert_eq!(decoded_pos, SwapMode::Position);
    }
}
