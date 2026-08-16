use ipc_protocol::DiscoveredWidgetInfo;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use widget_parser::WidgetManifest;

/// Dynamic filesystem scanner that recursively searches directories for `widget.toml` manifests.
pub struct WidgetDiscoveryScanner;

impl WidgetDiscoveryScanner {
    /// Recursively scans search directories for `widget.toml` manifests.
    pub fn scan_directories(search_paths: &[PathBuf], loaded_widget_ids: &[String]) -> Vec<DiscoveredWidgetInfo> {
        let mut discovered = Vec::new();
        let mut visited_paths = std::collections::HashSet::new();

        for root in search_paths {
            if !root.exists() {
                continue;
            }
            Self::scan_recursive(root, &mut discovered, &mut visited_paths, loaded_widget_ids);
        }

        info!("WidgetDiscoveryScanner: Found {} valid widget manifests on disk", discovered.len());
        discovered
    }

    /// Recursively walks a directory tree searching for `widget.toml`.
    fn scan_recursive(
        dir: &Path,
        discovered: &mut Vec<DiscoveredWidgetInfo>,
        visited: &mut std::collections::HashSet<PathBuf>,
        loaded_widget_ids: &[String],
    ) {
        let Ok(entries) = fs::read_dir(dir) else { return };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden folders, build target outputs, and dependency trees
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" || file_name == "bin" || file_name == "obj" {
                        continue;
                    }
                }
                Self::scan_recursive(&path, discovered, visited, loaded_widget_ids);
            } else if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.eq_ignore_ascii_case("widget.toml") {
                        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                        if visited.insert(canonical) {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(manifest) = WidgetManifest::parse_toml(&content) {
                                    let is_loaded = loaded_widget_ids.contains(&manifest.metadata.id);
                                    let folder_path = path.parent().unwrap_or(&path).to_string_lossy().to_string();
                                    let manifest_path = path.to_string_lossy().to_string();

                                    discovered.push(DiscoveredWidgetInfo {
                                        id: manifest.metadata.id,
                                        name: manifest.metadata.name,
                                        author: manifest.metadata.author,
                                        version: manifest.metadata.version,
                                        update_interval_ms: manifest.metadata.update_interval_ms,
                                        manifest_path,
                                        folder_path,
                                        is_loaded,
                                        is_locked: false,
                                        position_x: 100,
                                        position_y: 100,
                                        target_fps: 60,
                                        description: format!(
                                            "Layout: {:.0}x{:.0} px | {} elements",
                                            manifest.layout.width, manifest.layout.height, manifest.elements.len()
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_discovery_scanner_scans_workspace() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let search_paths = vec![root];
        let loaded = vec!["aether.builtin.perf_monitor".to_string()];

        let discovered = WidgetDiscoveryScanner::scan_directories(&search_paths, &loaded);
        assert!(!discovered.is_empty(), "Should discover built-in widget manifests in crates/");

        let perf_widget = discovered.iter().find(|w| w.id == "aether.builtin.perf_monitor");
        assert!(perf_widget.is_some());
        assert!(perf_widget.unwrap().is_loaded);
    }
}
