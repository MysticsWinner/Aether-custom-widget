use crate::resolver::{DynamicThemeStore, ThemeResolver};
use crate::schema::ThemeSchema;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// File watcher and dynamic reloader for `theme.json` files.
/// Enables real-time hot reloading without restarting the host daemon process.
pub struct ThemeWatcher {
    theme_file_path: PathBuf,
    theme_store: DynamicThemeStore,
}

impl ThemeWatcher {
    pub fn new(file_path: PathBuf, store: DynamicThemeStore) -> Self {
        Self {
            theme_file_path: file_path,
            theme_store: store,
        }
    }

    /// Loads and parses `theme.json` from disk, applying it directly to the active theme store.
    pub fn reload_now(&self) -> anyhow::Result<()> {
        if !self.theme_file_path.exists() {
            return Err(anyhow::anyhow!("Theme file path does not exist: {:?}", self.theme_file_path));
        }

        info!("Reading and parsing theme configuration from: {:?}", self.theme_file_path);
        let content = fs::read_to_string(&self.theme_file_path)?;
        let schema = ThemeSchema::parse_json(&content)?;

        self.theme_store.hot_swap_schema(schema);
        info!("Hot reload completed successfully for {:?}", self.theme_file_path);
        Ok(())
    }

    /// Returns the target watched theme file path.
    pub fn file_path(&self) -> &Path {
        &self.theme_file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_watcher_reload() {
        let store = DynamicThemeStore::default();
        let temp_dir = std::env::temp_dir();
        let test_theme_file = temp_dir.join("test_theme.json");

        let schema = ThemeSchema::default();
        let json_str = schema.to_json().unwrap();
        fs::write(&test_theme_file, json_str).unwrap();

        let watcher = ThemeWatcher::new(test_theme_file.clone(), store.clone());
        assert!(watcher.reload_now().is_ok());

        // Cleanup
        let _ = fs::remove_file(test_theme_file);
    }
}
