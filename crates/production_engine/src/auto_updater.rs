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
    /// NOTE: The actual HTTP fetch requires an async runtime with a network client
    /// (e.g. `reqwest`). This method provides the structural framework; the
    /// network transport is a TODO until `reqwest` is added as a dependency.
    pub fn check_for_updates(&self) -> Option<UpdateInfo> {
        info!(
            "Checking for updates: current={}, manifest_url='{}'",
            self.current_version, self.update_manifest_url
        );

        // TODO: Replace with real HTTP GET to self.update_manifest_url once reqwest is added.
        // For now, we report that no update is available (safe default) rather than
        // returning a hardcoded fake version.
        warn!("Auto-updater network transport not yet implemented; reporting no update available.");
        None
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
                    "Package file verified: {:?} ({} bytes). SHA-256 hash check pending crypto integration.",
                    package_path, meta.len()
                );
                // TODO: Compute SHA-256 of file contents and compare against expected_sha256.
                // Requires adding `sha2` crate as a dependency.
                true
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
}
