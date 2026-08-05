use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::info;

/// Custom screen coordinates and lock status for a rendered widget plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WidgetPositionSpec {
    pub x: i32,
    pub y: i32,
    pub is_locked: bool,
}

/// Persistent user configuration position store for rendered widgets.
///
/// Ensures user position adjustments (drag & drop) and lock/unlock states
/// are saved to disk without overwriting default TOML manifests.
#[derive(Debug, Clone)]
pub struct WidgetPositionStore {
    file_path: Option<PathBuf>,
    positions: Arc<RwLock<HashMap<String, WidgetPositionSpec>>>,
}

impl WidgetPositionStore {
    /// Creates a new position store with an optional JSON persistence path.
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let store = Self {
            file_path,
            positions: Arc::new(RwLock::new(HashMap::new())),
        };
        let _ = store.load_from_disk();
        store
    }

    /// Creates an in-memory position store (for testing or transient layouts).
    pub fn in_memory() -> Self {
        Self::new(None)
    }

    /// Returns custom (x, y) coordinates for a widget if saved.
    pub fn get_position(&self, widget_id: &str) -> Option<(i32, i32)> {
        self.positions
            .read()
            .ok()
            .and_then(|map| map.get(widget_id).map(|pos| (pos.x, pos.y)))
    }

    /// Updates or inserts custom (x, y) position for a widget and persists to disk.
    pub fn set_position(&self, widget_id: &str, x: i32, y: i32) -> anyhow::Result<()> {
        if let Ok(mut map) = self.positions.write() {
            if let Some(pos) = map.get_mut(widget_id) {
                pos.x = x;
                pos.y = y;
            } else {
                map.insert(
                    widget_id.to_string(),
                    WidgetPositionSpec {
                        x,
                        y,
                        is_locked: false,
                    },
                );
            }
        }
        self.save_to_disk()
    }

    /// Returns true if the widget position is locked against dragging.
    pub fn is_locked(&self, widget_id: &str) -> bool {
        self.positions
            .read()
            .ok()
            .and_then(|map| map.get(widget_id).map(|pos| pos.is_locked))
            .unwrap_or(false)
    }

    /// Sets lock status for a widget (true = locked, false = drag enabled).
    pub fn set_locked(&self, widget_id: &str, locked: bool) -> anyhow::Result<()> {
        if let Ok(mut map) = self.positions.write() {
            if let Some(pos) = map.get_mut(widget_id) {
                pos.is_locked = locked;
            } else {
                map.insert(
                    widget_id.to_string(),
                    WidgetPositionSpec {
                        x: 0,
                        y: 0,
                        is_locked: locked,
                    },
                );
            }
        }
        self.save_to_disk()
    }

    /// Toggles lock status for a widget and returns the new state.
    pub fn toggle_locked(&self, widget_id: &str) -> bool {
        let new_state = !self.is_locked(widget_id);
        let _ = self.set_locked(widget_id, new_state);
        new_state
    }

    /// Resets custom position and lock status for a widget, restoring default manifest position.
    pub fn reset_position(&self, widget_id: &str) -> bool {
        let removed = if let Ok(mut map) = self.positions.write() {
            map.remove(widget_id).is_some()
        } else {
            false
        };
        let _ = self.save_to_disk();
        removed
    }

    /// Loads custom widget positions from disk if file exists.
    pub fn load_from_disk(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.file_path {
            if path.exists() {
                let content = fs::read_to_string(path)?;
                let map: HashMap<String, WidgetPositionSpec> = serde_json::from_str(&content)?;
                if let Ok(mut lock) = self.positions.write() {
                    *lock = map;
                }
                info!("Loaded {} custom widget positions from '{:?}'", self.positions.read().map(|m| m.len()).unwrap_or(0), path);
            }
        }
        Ok(())
    }

    /// Persists custom widget positions to disk.
    pub fn save_to_disk(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.file_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            if let Ok(map) = self.positions.read() {
                let json = serde_json::to_string_pretty(&*map)?;
                fs::write(path, json)?;
            }
        }
        Ok(())
    }
}

impl Default for WidgetPositionStore {
    fn default() -> Self {
        let default_path = Path::new(".aether").join("widget_positions.json");
        Self::new(Some(default_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_position_store_set_get() {
        let store = WidgetPositionStore::in_memory();
        assert_eq!(store.get_position("perf_monitor_widget"), None);

        store.set_position("perf_monitor_widget", 400, 200).unwrap();
        assert_eq!(store.get_position("perf_monitor_widget"), Some((400, 200)));
        assert!(!store.is_locked("perf_monitor_widget"));

        store.set_locked("perf_monitor_widget", true).unwrap();
        assert!(store.is_locked("perf_monitor_widget"));

        let toggled = store.toggle_locked("perf_monitor_widget");
        assert!(!toggled);
        assert!(!store.is_locked("perf_monitor_widget"));
    }

    #[test]
    fn test_position_store_json_persistence() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("widget_positions.json");

        let store = WidgetPositionStore::new(Some(file_path.clone()));
        store.set_position("plugin_a", 150, 350).unwrap();
        store.set_locked("plugin_a", true).unwrap();

        assert!(file_path.exists());

        // Reload from file in a second store instance
        let store2 = WidgetPositionStore::new(Some(file_path));
        assert_eq!(store2.get_position("plugin_a"), Some((150, 350)));
        assert!(store2.is_locked("plugin_a"));
    }

    #[test]
    fn test_position_store_reset() {
        let store = WidgetPositionStore::in_memory();
        store.set_position("plugin_b", 500, 500).unwrap();
        assert_eq!(store.get_position("plugin_b"), Some((500, 500)));

        assert!(store.reset_position("plugin_b"));
        assert_eq!(store.get_position("plugin_b"), None);
    }
}
