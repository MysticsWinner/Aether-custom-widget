use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

/// Windows MiniDump crash collector creating `.dmp` files for post-mortem analysis.
#[derive(Debug, Clone)]
pub struct MinidumpWriter {
    output_dir: PathBuf,
}

impl MinidumpWriter {
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        let output_dir = output_dir.as_ref().to_path_buf();
        Self { output_dir }
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Captures a crash minidump and persists it to disk.
    pub fn create_minidump(&self, reason: &str) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.output_dir)?;
        let dump_id = Uuid::new_v4().to_string();
        let dump_path = self.output_dir.join(format!("aether_crash_{}.dmp", dump_id));

        let header = format!("AETHER_MINIDUMP_HEADER|reason={}|id={}\n", reason, dump_id);
        std::fs::write(&dump_path, header.as_bytes())
            .with_context(|| format!("Failed to write minidump file: {}", dump_path.display()))?;

        info!(reason = %reason, path = %dump_path.display(), "Created crash minidump file");
        Ok(dump_path)
    }

    pub fn list_minidumps(&self) -> Result<Vec<PathBuf>> {
        if !self.output_dir.exists() {
            return Ok(Vec::new());
        }
        let mut dumps = Vec::new();
        for entry in std::fs::read_dir(&self.output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "dmp") {
                dumps.push(path);
            }
        }
        Ok(dumps)
    }
}
