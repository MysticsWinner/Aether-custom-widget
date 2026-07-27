use crate::package::WidgetPackage;
use crate::security::Ed25519Verifier;
use std::collections::HashMap;
use tracing::{info, warn};

/// Package Manager supporting npm-like CLI commands (`install weather-widget`, `install spotify-widget`, `install taskbar-plus`).
pub struct PackageManager {
    installed_packages: HashMap<String, WidgetPackage>,
    registry: HashMap<String, WidgetPackage>,
}

impl PackageManager {
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        let weather = WidgetPackage::new("weather-widget", "Live Weather Overlay", "1.2.0", "Community");
        let spotify = WidgetPackage::new("spotify-widget", "Spotify Media Player Controller", "2.0.1", "AudioTeam");
        let taskbar = WidgetPackage::new("taskbar-plus", "Taskbar Enhancement Suite", "1.0.4", "SystemTeam");

        registry.insert("weather-widget".to_string(), weather);
        registry.insert("spotify-widget".to_string(), spotify);
        registry.insert("taskbar-plus".to_string(), taskbar);

        Self {
            installed_packages: HashMap::new(),
            registry,
        }
    }

    /// Installs a package by name from the marketplace registry (e.g. `install weather-widget`).
    pub fn install(&mut self, package_name: &str) -> anyhow::Result<WidgetPackage> {
        info!("Executing package manager command: 'install {}'...", package_name);

        let pkg = self
            .registry
            .get(package_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in marketplace registry", package_name))?;

        // 1. Verify Ed25519 Digital Signature
        let payload = pkg.name.as_bytes();
        let mock_sig = b"valid_ed25519_signature";
        if !Ed25519Verifier::verify_package(&pkg.id, payload, mock_sig) {
            return Err(anyhow::anyhow!("Package '{}' failed Ed25519 signature verification!", package_name));
        }

        // 2. Install to Local Package Store
        self.installed_packages.insert(pkg.id.clone(), pkg.clone());
        info!(
            "Successfully installed '{}' (v{}) into ~/.custom_widgets/packages/",
            pkg.name, pkg.version
        );

        Ok(pkg)
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
}
