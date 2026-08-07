use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;

/// File system watcher monitoring widget directories for live hot-reload triggers.
#[derive(Debug, Clone)]
pub struct DevHotReloader {
    watch_paths: Vec<PathBuf>,
    pending_reloads: HashSet<String>,
}

impl DevHotReloader {
    pub fn new() -> Self {
        Self {
            watch_paths: Vec::new(),
            pending_reloads: HashSet::new(),
        }
    }

    pub fn watch_directory<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref().to_path_buf();
        info!(path = %p.display(), "Added directory to hot-reload watcher");
        self.watch_paths.push(p);
    }

    pub fn notify_file_change<P: AsRef<Path>>(&mut self, file_path: P) -> Option<String> {
        let path = file_path.as_ref();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext == "lua" || ext == "toml" {
            let widget_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown_widget")
                .to_string();

            info!(widget_id = %widget_id, file = %path.display(), "Hot-reload file change detected");
            self.pending_reloads.insert(widget_id.clone());
            Some(widget_id)
        } else {
            None
        }
    }

    pub fn drain_reloads(&mut self) -> Vec<String> {
        self.pending_reloads.drain().collect()
    }
}

impl Default for DevHotReloader {
    fn default() -> Self {
        Self::new()
    }
}
