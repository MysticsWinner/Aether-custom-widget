//! Next-Gen Windows Desktop Customization Platform - Theme Engine Crate
//!
//! Provides dynamic theme token resolution driven by declarative `theme.json` files
//! supporting colors, fonts, icons, widgets, layouts, and animations—with live Hot Reloading without host restart.

pub mod accessibility;
pub mod dynamic_color;
pub mod hot_reload;
pub mod material;
pub mod motion;
pub mod resolver;
pub mod schema;
pub mod tokens;
pub mod typography;

pub use accessibility::{AccessibilityConfig, AccessibilityEngine};
pub use dynamic_color::{DynamicColorEngine, DynamicPalette};
pub use hot_reload::ThemeWatcher;
pub use material::{MaterialContext, MaterialEngine, MaterialSpec, MaterialType};
pub use motion::{MotionEngine, MotionLevel, MotionSpec};
pub use resolver::{DynamicThemeStore, ThemeBenchmark, ThemeResolver};
pub use schema::{AnimationConfig, FontConfig, LayoutConfig, ThemeMetadata, ThemeSchema};
pub use tokens::DesignTokens;
pub use typography::{TypographyEngine, TypographyRole, TypographySpec};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemThemeMode {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct ThemeManager {
    store: DynamicThemeStore,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            store: DynamicThemeStore::default(),
        }
    }

    pub fn set_mode(&mut self, mode: SystemThemeMode) {
        let mut schema = ThemeSchema::default();
        match mode {
            SystemThemeMode::Dark => {
                schema.colors.insert("theme.background".into(), "#1E1E1EE6".into());
                schema.colors.insert("theme.text_primary".into(), "#FFFFFF".into());
            }
            SystemThemeMode::Light => {
                schema.colors.insert("theme.background".into(), "#F5F5F5E6".into());
                schema.colors.insert("theme.text_primary".into(), "#0F0F0F".into());
            }
        }
        self.store.hot_swap_schema(schema);
    }

    pub fn store(&self) -> DynamicThemeStore {
        self.store.clone()
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
