//! Platform Accessibility Engine
//!
//! Centralized access control and global override system enforcing High Contrast,
//! Reduce Motion, Reduce Transparency, and Large Text across theme, material, typography,
//! and layout engines.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessibilityConfig {
    pub high_contrast: bool,
    pub reduce_motion: bool,
    pub reduce_transparency: bool,
    pub text_scale: f32,
    pub focus_indicator_width: f32,
    pub forced_contrast_palette: Option<Vec<String>>,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            high_contrast: false,
            reduce_motion: false,
            reduce_transparency: false,
            text_scale: 1.0,
            focus_indicator_width: 2.0,
            forced_contrast_palette: None,
        }
    }
}

pub struct AccessibilityEngine {
    config: AccessibilityConfig,
}

impl AccessibilityEngine {
    pub fn new(config: AccessibilityConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &AccessibilityConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: AccessibilityConfig) {
        self.config = new_config;
    }

    /// Checks if any accessibility override is active.
    pub fn is_any_override_active(&self) -> bool {
        self.config.high_contrast
            || self.config.reduce_motion
            || self.config.reduce_transparency
            || (self.config.text_scale - 1.0).abs() > 0.05
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_engine_overrides() {
        let mut engine = AccessibilityEngine::new(AccessibilityConfig::default());
        assert!(!engine.is_any_override_active());

        engine.update_config(AccessibilityConfig {
            high_contrast: true,
            ..Default::default()
        });
        assert!(engine.is_any_override_active());
    }
}
