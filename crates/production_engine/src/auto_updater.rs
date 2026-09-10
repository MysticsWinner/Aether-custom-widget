use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Represents a discovered update with version, download URL, and expected hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub sha256_hash: String,
    pub release_notes: String,
}

/// Delta MSIX Auto-Updater Engine.
///
/// Compares the current running version against a release manifest to determine
/// whether an update is available. Signature verification uses SHA-256 hash
/// comparison of the downloaded package against the manifest-declared hash.
pub struct AutoUpdater {
    current_version: String,
    update_manifest_url: String,
    download_dir: PathBuf,
}

impl AutoUpdater {
    /// Creates a new updater for the given current version.
    pub fn new(current_version: &str) -> Self {
        let download_dir = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("Aether")
            .join("updates");
        Self {
            current_version: current_version.to_string(),
            update_manifest_url: "https://api.github.com/repos/MysticsWinner/Aether-custom-widget/releases/latest".to_string(),
            download_dir,
        }
    }

    /// Creates an updater with a custom manifest URL (useful for testing / enterprise).
    pub fn with_manifest_url(current_version: &str, manifest_url: &str) -> Self {
        let mut updater = Self::new(current_version);
        updater.update_manifest_url = manifest_url.to_string();
        updater
    }

    /// Returns the current running version.
    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    /// Returns the configured update manifest URL.
    pub fn manifest_url(&self) -> &str {
        &self.update_manifest_url
    }

    /// Checks for available updates by comparing current version against a release manifest.
    ///
    /// Returns `Some(UpdateInfo)` if a newer version exists, `None` otherwise.
    /// Network transport performs a synchronous HTTP GET via `reqwest::blocking` to fetch
    /// and deserialize the release manifest JSON.
    pub fn check_for_updates(&self) -> Option<UpdateInfo> {
        info!(
            "Checking for updates: current={}, manifest_url='{}'",
            self.current_version, self.update_manifest_url
        );

        // Perform a blocking HTTP GET to fetch the release manifest JSON.
        // The manifest is expected to contain fields: version, download_url, sha256_hash, release_notes.
        // This implementation uses `reqwest::blocking` to keep the API sync.
        match reqwest::blocking::get(&self.update_manifest_url) {
            Ok(resp) => {
                if let Ok(text) = resp.text() {
                    #[derive(serde::Deserialize)]
                    struct Manifest {
                        version: String,
                        download_url: String,
                        sha256_hash: String,
                        release_notes: String,
                    }
                    match serde_json::from_str::<Manifest>(&text) {
                        Ok(manifest) => {
                            if self.is_newer_version(&manifest.version) {
                                info!("Update available: {}", manifest.version);
                                return Some(UpdateInfo {
                                    version: manifest.version,
                                    download_url: manifest.download_url,
                                    sha256_hash: manifest.sha256_hash,
                                    release_notes: manifest.release_notes,
                                });
                            } else {
                                info!("No newer version found.");
                                return None;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse update manifest JSON: {}", e);
                            return None;
                        }
                    }
                } else {
                    warn!("Failed to read response body from update manifest URL.");
                    return None;
                }
            }
            Err(e) => {
                warn!("Network error while checking for updates: {}", e);
                return None;
            }
        }
    }

    /// Downloads the update package from `update.download_url` into `self.download_dir`,
    /// verifies its SHA-256 hash against `update.sha256_hash`, and returns the local path.
    /// If verification fails, the downloaded file is removed and an error is returned.
    pub fn download_package(&self, update: &UpdateInfo) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.download_dir)?;
        let file_name = Path::new(&update.download_url)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("update.msix");
        let dest_path = self.download_dir.join(format!("{}_{}", update.version, file_name));

        info!("Downloading update package from '{}' to {:?}", update.download_url, dest_path);

        let mut response = reqwest::blocking::get(&update.download_url)
            .map_err(|e| anyhow::anyhow!("Failed to download update package: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error downloading update package: {}", response.status()));
        }

        let mut file = std::fs::File::create(&dest_path)?;
        std::io::copy(&mut response, &mut file)?;
        drop(file);

        if !Self::verify_package_integrity(&dest_path, &update.sha256_hash) {
            let _ = std::fs::remove_file(&dest_path);
            return Err(anyhow::anyhow!("Integrity check failed: package hash mismatch"));
        }

        info!("Update package successfully downloaded and verified at {:?}", dest_path);
        Ok(dest_path)
    }

    /// Verifies a downloaded package by checking it exists and has non-zero size.
    /// Full verification requires SHA-256 hash comparison against manifest-declared hash.
    pub fn verify_package_integrity(package_path: &Path, _expected_sha256: &str) -> bool {
        if !package_path.exists() {
            warn!("Package verification failed: file does not exist at {:?}", package_path);
            return false;
        }

        match std::fs::metadata(package_path) {
            Ok(meta) if meta.len() == 0 => {
                warn!("Package verification failed: file is empty at {:?}", package_path);
                false
            }
            Ok(meta) => {
                info!(
                    "Package file verified: {:?} ({} bytes). Performing SHA-256 hash verification...",
                    package_path, meta.len()
                );
                // Compute SHA-256 of the file contents and compare with the expected hash.
                use sha2::{Digest, Sha256};
                let mut file = match std::fs::File::open(package_path) {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("Unable to open file for hash verification: {}", e);
                        return false;
                    }
                };
                let mut hasher = Sha256::new();
                if std::io::copy(&mut file, &mut hasher).is_ok() {
                    let result = hasher.finalize();
                    let computed_hash = format!("{:x}", result);
                    if computed_hash.eq_ignore_ascii_case(_expected_sha256) {
                        info!("Package hash verified successfully.");
                        true
                    } else {
                        warn!("Package hash mismatch. Expected {}, got {}.", _expected_sha256, computed_hash);
                        false
                    }
                } else {
                    warn!("Failed to read file contents for hash verification.");
                    false
                }
            }
            Err(e) => {
                warn!("Package verification failed: cannot read metadata for {:?}: {}", package_path, e);
                false
            }
        }
    }

    /// Returns the download directory for update packages.
    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }

    /// Checks whether a given version string is newer than the current version.
    /// Uses simple semver comparison (major.minor.patch).
    pub fn is_newer_version(&self, candidate: &str) -> bool {
        let parse = |v: &str| -> Option<(u32, u32, u32)> {
            let parts: Vec<&str> = v.split('.').collect();
            if parts.len() >= 3 {
                Some((
                    parts[0].parse().ok()?,
                    parts[1].parse().ok()?,
                    parts[2].parse().ok()?,
                ))
            } else {
                None
            }
        };

        match (parse(&self.current_version), parse(candidate)) {
            (Some(cur), Some(cand)) => cand > cur,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_updater_reports_no_update_without_network() {
        let updater = AutoUpdater::new("0.6.0");
        // Without network transport, should safely report None
        assert_eq!(updater.check_for_updates(), None);
    }

    #[test]
    fn test_auto_updater_version_comparison() {
        let updater = AutoUpdater::new("0.6.0");
        assert!(updater.is_newer_version("0.7.0"));
        assert!(updater.is_newer_version("1.0.0"));
        assert!(updater.is_newer_version("0.6.1"));
        assert!(!updater.is_newer_version("0.6.0"));
        assert!(!updater.is_newer_version("0.5.9"));
    }

    #[test]
    fn test_auto_updater_custom_manifest_url() {
        let updater = AutoUpdater::with_manifest_url("1.0.0", "https://example.com/updates.json");
        assert_eq!(updater.manifest_url(), "https://example.com/updates.json");
        assert_eq!(updater.current_version(), "1.0.0");
    }

    #[test]
    fn test_auto_updater_verify_nonexistent_package() {
        let result = AutoUpdater::verify_package_integrity(
            Path::new("nonexistent.msix"),
            "abc123",
        );
        assert!(!result);
    }

    #[test]
    fn test_auto_updater_download_dir_is_valid() {
        let updater = AutoUpdater::new("0.6.0");
        let dir = updater.download_dir();
        assert!(dir.to_string_lossy().contains("Aether"));
    }

    #[test]
    fn test_auto_updater_integrity_verification_sha256() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir().join("aether_test_update_sha");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test_pkg.bin");
        let payload = b"Aether Update Binary Payload Test 12345";
        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(payload).unwrap();
        drop(file);

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let expected_hash = format!("{:x}", hasher.finalize());

        assert!(AutoUpdater::verify_package_integrity(&test_file, &expected_hash));
        assert!(!AutoUpdater::verify_package_integrity(
            &test_file,
            "0000000000000000000000000000000000000000000000000000000000000000"
        ));

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_auto_updater_download_package_unreachable_url() {
        let updater = AutoUpdater::new("0.6.0");
        let update_info = UpdateInfo {
            version: "0.7.0".to_string(),
            download_url: "http://127.0.0.1:9/nonexistent.msix".to_string(),
            sha256_hash: "abcd".to_string(),
            release_notes: "notes".to_string(),
        };
        let result = updater.download_package(&update_info);
        assert!(result.is_err());
    }
}
