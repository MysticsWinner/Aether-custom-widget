// core_engine/src/widget_config_store.rs
// Copyright (c) Aether Platform. Licensed under the MIT License.
//
// In-process store for per-widget display configs.  Each widget's config is
// persisted as an individual JSON file under:
//   %TEMP%\aether_widgets\<widget_id>.json
//
// The store is Arc<Mutex<WidgetConfigStore>> in IpcSharedState so any IPC
// handler can read or update configs safely from async tasks.

use std::collections::HashMap;
use std::path::PathBuf;

use serde_json;
use tracing::{info, warn};
use widget_sdk::{ColourOverrides, DisplayOptions, WidgetConfig};

pub struct WidgetConfigStore {
    configs: HashMap<String, WidgetConfig>,
    base_dir: PathBuf,
}

impl WidgetConfigStore {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            configs: HashMap::new(),
            base_dir,
        }
    }

    /// Return a clone of the config for `widget_id`, creating a default one if absent.
    pub fn get_or_default(&mut self, widget_id: &str) -> WidgetConfig {
        if !self.configs.contains_key(widget_id) {
            let cfg = self.load_from_disk(widget_id).unwrap_or_else(|| WidgetConfig::new(widget_id));
            self.configs.insert(widget_id.to_string(), cfg);
        }
        self.configs[widget_id].clone()
    }

    /// Update the display options for a widget and persist immediately.
    pub fn update_display_options(
        &mut self,
        widget_id: &str,
        opacity: Option<f32>,
        scale: Option<f32>,
        locked: Option<bool>,
        enabled: Option<bool>,
    ) {
        let cfg = self.get_or_default(widget_id);
        let updated = WidgetConfig {
            widget_id: widget_id.to_string(),
            display_options: DisplayOptions {
                opacity: opacity.unwrap_or(cfg.display_options.opacity).clamp(0.0, 1.0),
                scale: scale.unwrap_or(cfg.display_options.scale).max(0.1),
                locked: locked.unwrap_or(cfg.display_options.locked),
                enabled: enabled.unwrap_or(cfg.display_options.enabled),
            },
            colour_overrides: cfg.colour_overrides.clone(),
            quick_swap: cfg.quick_swap,
        };
        self.configs.insert(widget_id.to_string(), updated.clone());
        self.persist(&updated);
    }

    /// Set quick_swap flag for a widget.
    pub fn set_quick_swap(&mut self, widget_id: &str, quick_swap: bool) {
        let mut cfg = self.get_or_default(widget_id);
        cfg.quick_swap = quick_swap;
        self.configs.insert(widget_id.to_string(), cfg.clone());
        self.persist(&cfg);
    }

    /// Set colour overrides for a widget.
    pub fn set_colour_overrides(&mut self, widget_id: &str, overrides: Option<ColourOverrides>) {
        let mut cfg = self.get_or_default(widget_id);
        cfg.colour_overrides = overrides;
        self.configs.insert(widget_id.to_string(), cfg.clone());
        self.persist(&cfg);
    }

    /// Reset a widget to default config.
    pub fn reset(&mut self, widget_id: &str) {
        let cfg = WidgetConfig::new(widget_id);
        self.configs.insert(widget_id.to_string(), cfg.clone());
        self.persist(&cfg);
    }

    /// Swap configs between two widgets (configuration mode).
    pub fn swap_configs(&mut self, from_id: &str, to_id: &str) {
        let from = self.get_or_default(from_id);
        let to = self.get_or_default(to_id);

        let updated_from = WidgetConfig {
            widget_id: from_id.to_string(),
            display_options: to.display_options.clone(),
            colour_overrides: to.colour_overrides.clone(),
            quick_swap: to.quick_swap,
        };
        let updated_to = WidgetConfig {
            widget_id: to_id.to_string(),
            display_options: from.display_options,
            colour_overrides: from.colour_overrides,
            quick_swap: from.quick_swap,
        };

        self.configs.insert(from_id.to_string(), updated_from.clone());
        self.configs.insert(to_id.to_string(), updated_to.clone());
        self.persist(&updated_from);
        self.persist(&updated_to);
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn file_path(&self, widget_id: &str) -> PathBuf {
        let safe_id = widget_id.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
        self.base_dir.join(format!("{safe_id}.json"))
    }

    fn persist(&self, cfg: &WidgetConfig) {
        let path = self.file_path(&cfg.widget_id);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(cfg) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    warn!("WidgetConfigStore: failed to persist '{}': {e}", cfg.widget_id);
                } else {
                    info!("WidgetConfigStore: persisted config for '{}'", cfg.widget_id);
                }
            }
            Err(e) => warn!("WidgetConfigStore: serialize error for '{}': {e}", cfg.widget_id),
        }
    }

    fn load_from_disk(&self, widget_id: &str) -> Option<WidgetConfig> {
        let path = self.file_path(widget_id);
        let json = std::fs::read_to_string(&path).ok()?;
        match serde_json::from_str::<WidgetConfig>(&json) {
            Ok(cfg) => {
                info!("WidgetConfigStore: loaded config for '{}' from disk", widget_id);
                Some(cfg)
            }
            Err(e) => {
                warn!("WidgetConfigStore: corrupt config for '{widget_id}': {e}");
                None
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> WidgetConfigStore {
        let dir = std::env::temp_dir().join("aether_widget_cfg_test");
        WidgetConfigStore::new(dir)
    }

    #[test]
    fn test_widget_config_store_default_creation() {
        let mut store = temp_store();
        let cfg = store.get_or_default("perf_monitor");
        assert_eq!(cfg.widget_id, "perf_monitor");
        assert!((cfg.display_options.opacity - 1.0).abs() < f32::EPSILON);
        assert!(cfg.display_options.enabled);
    }

    #[test]
    fn test_widget_config_store_update_opacity() {
        let mut store = temp_store();
        store.update_display_options("w1", Some(0.5), None, None, None);
        let cfg = store.get_or_default("w1");
        assert!((cfg.display_options.opacity - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_widget_config_store_opacity_clamped() {
        let mut store = temp_store();
        store.update_display_options("w2", Some(2.5), None, None, None);
        let cfg = store.get_or_default("w2");
        assert!((cfg.display_options.opacity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_widget_config_store_reset() {
        let mut store = temp_store();
        store.update_display_options("w3", Some(0.2), None, Some(true), None);
        store.reset("w3");
        let cfg = store.get_or_default("w3");
        assert!((cfg.display_options.opacity - 1.0).abs() < f32::EPSILON);
        assert!(!cfg.display_options.locked);
    }

    #[test]
    fn test_widget_config_store_swap_configs() {
        let mut store = temp_store();
        store.update_display_options("wa", Some(0.3), None, None, None);
        store.update_display_options("wb", Some(0.9), None, None, None);
        store.swap_configs("wa", "wb");
        let wa = store.get_or_default("wa");
        let wb = store.get_or_default("wb");
        assert!((wa.display_options.opacity - 0.9).abs() < f32::EPSILON);
        assert!((wb.display_options.opacity - 0.3).abs() < f32::EPSILON);
    }
}
