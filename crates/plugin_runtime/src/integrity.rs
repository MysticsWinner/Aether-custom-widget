use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Computes BLAKE3 hash of a binary file.
pub fn compute_blake3_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let bytes = std::fs::read(path.as_ref())
        .with_context(|| format!("Failed to read binary for hashing: {}", path.as_ref().display()))?;
    let hash = blake3::hash(&bytes);
    Ok(hash.to_hex().to_string())
}

/// Persistent store mapping plugin ID to verified BLAKE3 binary hash.
#[derive(Debug, Clone)]
pub struct PluginHashStore {
    store_path: PathBuf,
    hashes: HashMap<String, String>,
}

impl PluginHashStore {
    pub fn new<P: AsRef<Path>>(store_path: P) -> Self {
        let store_path = store_path.as_ref().to_path_buf();
        let hashes = Self::load_from_disk(&store_path).unwrap_or_default();
        Self { store_path, hashes }
    }

    fn load_from_disk(path: &Path) -> Result<HashMap<String, String>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(path)?;
        let map: HashMap<String, String> = serde_json::from_str(&content)?;
        Ok(map)
    }

    pub fn save_to_disk(&self) -> Result<()> {
        if let Some(parent) = self.store_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.hashes)?;
        std::fs::write(&self.store_path, content)?;
        Ok(())
    }

    pub fn register_hash(&mut self, plugin_id: &str, hash: &str) -> Result<()> {
        info!(plugin_id = %plugin_id, hash = %hash, "Registering trusted plugin BLAKE3 binary hash");
        self.hashes.insert(plugin_id.to_string(), hash.to_string());
        self.save_to_disk()
    }

    pub fn get_hash(&self, plugin_id: &str) -> Option<&String> {
        self.hashes.get(plugin_id)
    }

    pub fn verify_plugin_binary<P: AsRef<Path>>(&self, plugin_id: &str, binary_path: P) -> Result<bool> {
        let expected_hash = match self.get_hash(plugin_id) {
            Some(h) => h,
            None => {
                info!(plugin_id = %plugin_id, "No prior hash registered for plugin; allowing initial load.");
                return Ok(true);
            }
        };

        let actual_hash = compute_blake3_hash(binary_path)?;
        if actual_hash == *expected_hash {
            Ok(true)
        } else {
            warn!(
                plugin_id = %plugin_id,
                expected = %expected_hash,
                actual = %actual_hash,
                "BLAKE3 binary hash mismatch detected! Binary may be tampered."
            );
            Ok(false)
        }
    }
}
