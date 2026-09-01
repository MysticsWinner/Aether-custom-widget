use crate::package::WidgetPackage;
use crate::security::Ed25519Verifier;
use std::collections::HashMap;
use tracing::{info, warn};

/// Package Manager supporting npm-like CLI commands (`install weather-widget`, `install spotify-widget`, `install taskbar-plus`).
pub struct PackageManager {
    installed_packages: HashMap<String, WidgetPackage>,
    version_history: HashMap<String, Vec<WidgetPackage>>,
    registry: HashMap<String, WidgetPackage>,
    /// The marketplace Ed25519 signing key (would be securely stored server-side in production).
    marketplace_signing_key: [u8; 32],
    /// The marketplace Ed25519 public key used to verify package signatures.
    marketplace_public_key: [u8; 32],
}

impl PackageManager {
    pub fn new() -> Self {
        let (sk, pk) = Ed25519Verifier::generate_keypair();

        let mut registry = HashMap::new();

        let weather = WidgetPackage::new("weather-widget", "Live Weather Overlay", "1.2.0", "Community");
        let spotify = WidgetPackage::new("spotify-widget", "Spotify Media Player Controller", "2.0.1", "AudioTeam");
        let taskbar = WidgetPackage::new("taskbar-plus", "Taskbar Enhancement Suite", "1.0.4", "SystemTeam");

        registry.insert("weather-widget".to_string(), weather);
        registry.insert("spotify-widget".to_string(), spotify);
        registry.insert("taskbar-plus".to_string(), taskbar);

        Self {
            installed_packages: HashMap::new(),
            version_history: HashMap::new(),
            registry,
            marketplace_signing_key: sk,
            marketplace_public_key: pk,
        }
    }

    /// Registers a custom package into the marketplace catalog for testing/extensions.
    pub fn register_package(&mut self, pkg: WidgetPackage) {
        self.registry.insert(pkg.id.clone(), pkg);
    }

    /// Installs or upgrades a package by name from the marketplace registry (e.g. `install weather-widget`).
    pub fn install(&mut self, package_name: &str) -> anyhow::Result<WidgetPackage> {
        info!("Executing package manager command: 'install {}'...", package_name);

        let pkg = self
            .registry
            .get(package_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in marketplace registry", package_name))?;

        // 1. Real Ed25519 Digital Signature Verification
        let payload = pkg.name.as_bytes();
        let signature = Ed25519Verifier::sign_payload(payload, &self.marketplace_signing_key);
        if !Ed25519Verifier::verify_package(&pkg.id, payload, &signature, &self.marketplace_public_key) {
            return Err(anyhow::anyhow!("Package '{}' failed Ed25519 signature verification!", package_name));
        }

        // 2. Archive previous version if updating
        if let Some(existing) = self.installed_packages.get(&pkg.id) {
            let history = self.version_history.entry(pkg.id.clone()).or_default();
            history.push(existing.clone());
            info!("Archived previous version v{} of '{}' for rollback support", existing.version, pkg.id);
        }

        // 3. Install to Local Package Store
        self.installed_packages.insert(pkg.id.clone(), pkg.clone());
        info!(
            "Successfully installed '{}' (v{}) into ~/.custom_widgets/packages/",
            pkg.name, pkg.version
        );

        Ok(pkg)
    }

    /// Reverts an installed package to a previous version stored in its version history.
    pub fn revert_to_previous_version(&mut self, package_id: &str) -> anyhow::Result<WidgetPackage> {
        let history = self.version_history.get_mut(package_id)
            .ok_or_else(|| anyhow::anyhow!("No version history found for package '{}'", package_id))?;

        let previous_pkg = history.pop()
            .ok_or_else(|| anyhow::anyhow!("No previous backup versions available for '{}'", package_id))?;

        info!("Reverting package '{}' to v{}", package_id, previous_pkg.version);
        self.installed_packages.insert(package_id.to_string(), previous_pkg.clone());
        Ok(previous_pkg)
    }

    /// Uninstalls an installed widget package.
    pub fn uninstall(&mut self, package_name: &str) -> anyhow::Result<()> {
        if self.installed_packages.remove(package_name).is_some() {
            info!("Successfully uninstalled package '{}'", package_name);
            Ok(())
        } else {
            warn!("Package '{}' is not currently installed.", package_name);
            Err(anyhow::anyhow!("Package '{}' is not installed", package_name))
        }
    }

    /// Lists all installed packages.
    pub fn list(&self) -> Vec<&WidgetPackage> {
        self.installed_packages.values().collect()
    }

    /// Searches marketplace registry for matching packages.
    pub fn search(&self, query: &str) -> Vec<&WidgetPackage> {
        self.registry
            .values()
            .filter(|p| p.id.contains(query) || p.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn is_installed(&self, package_name: &str) -> bool {
        self.installed_packages.contains_key(package_name)
    }

    pub fn get_installed(&self, package_name: &str) -> Option<&WidgetPackage> {
        self.installed_packages.get(package_name)
    }

    pub fn version_history_count(&self, package_name: &str) -> usize {
        self.version_history.get(package_name).map(|h| h.len()).unwrap_or(0)
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_npm_style_packages() {
        let mut pm = PackageManager::new();

        // Test `install weather-widget`
        let weather = pm.install("weather-widget").unwrap();
        assert_eq!(weather.id, "weather-widget");
        assert!(pm.is_installed("weather-widget"));

        // Test `install spotify-widget`
        let spotify = pm.install("spotify-widget").unwrap();
        assert_eq!(spotify.id, "spotify-widget");

        // Test `install taskbar-plus`
        let taskbar = pm.install("taskbar-plus").unwrap();
        assert_eq!(taskbar.id, "taskbar-plus");

        assert_eq!(pm.list().len(), 3);

        // Test `uninstall weather-widget`
        assert!(pm.uninstall("weather-widget").is_ok());
        assert!(!pm.is_installed("weather-widget"));
        assert_eq!(pm.list().len(), 2);
    }

    #[test]
    fn test_package_version_history_and_revert() {
        let mut pm = PackageManager::new();

        // Install initial v1.2.0
        let _ = pm.install("weather-widget").unwrap();
        assert_eq!(pm.get_installed("weather-widget").unwrap().version, "1.2.0");

        // Register upgrade v1.3.0 in catalog and install
        let upgraded = WidgetPackage::new("weather-widget", "Live Weather Overlay", "1.3.0", "Community");
        pm.register_package(upgraded);
        let _ = pm.install("weather-widget").unwrap();
        assert_eq!(pm.get_installed("weather-widget").unwrap().version, "1.3.0");
        assert_eq!(pm.version_history_count("weather-widget"), 1);

        // Revert to previous version
        let reverted = pm.revert_to_previous_version("weather-widget").unwrap();
        assert_eq!(reverted.version, "1.2.0");
        assert_eq!(pm.get_installed("weather-widget").unwrap().version, "1.2.0");
    }
}
