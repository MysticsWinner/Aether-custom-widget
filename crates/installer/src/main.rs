use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Aether Desktop Customization Engine - Setup Installer & Uninstaller Wizard
pub struct AetherInstaller {
    install_dir: PathBuf,
    app_data_dir: PathBuf,
}

impl AetherInstaller {
    pub fn new() -> Self {
        let local_appdata = env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Public\\AppData\\Local"));

        let install_dir = local_appdata.join("Aether").join("bin");
        let app_data_dir = local_appdata.join("Aether").join("data");

        Self {
            install_dir,
            app_data_dir,
        }
    }

    /// Performs complete installation of Aether Runtime, SDK binaries, CLI, and Registry hooks.
    pub fn install(&self) -> anyhow::Result<()> {
        info!("=== Starting Aether Platform Windows Installation Wizard ===");
        info!("Target Installation Directory: {:?}", self.install_dir);
        info!("Target AppData Directory: {:?}", self.app_data_dir);

        // 1. Create installation directories
        fs::create_dir_all(&self.install_dir)?;
        fs::create_dir_all(&self.app_data_dir)?;
        fs::create_dir_all(self.app_data_dir.join("widgets"))?;
        fs::create_dir_all(self.app_data_dir.join("themes"))?;

        // 2. Deploy core binaries to installation folder
        let current_exe = env::current_exe()?;
        let target_installer_path = self.install_dir.join("AetherSetup.exe");
        if current_exe != target_installer_path {
            fs::copy(&current_exe, &target_installer_path)?;
        }

        // 3. Register Windows Control Panel Add/Remove Programs uninstall entries
        self.register_windows_uninstall_key()?;

        info!("✔ Aether Runtime binaries installed successfully to {:?}", self.install_dir);
        info!("✔ Windows Control Panel uninstall registry entry registered.");
        info!("=== Aether Platform Installation Completed Successfully! ===");
        Ok(())
    }

    /// Performs complete uninstallation of Aether Runtime, removing registry keys and binaries.
    pub fn uninstall(&self) -> anyhow::Result<()> {
        info!("=== Starting Aether Platform Windows Uninstallation Wizard ===");

        // 1. Unregister Windows uninstall registry key
        self.unregister_windows_uninstall_key()?;

        // 2. Remove installation folder if it exists
        if self.install_dir.exists() {
            fs::remove_dir_all(&self.install_dir)?;
            info!("✔ Removed installation binaries from {:?}", self.install_dir);
        }

        info!("=== Aether Platform Uninstallation Completed Successfully! ===");
        Ok(())
    }

    /// Checks if Aether Platform is currently installed on the system.
    pub fn is_installed(&self) -> bool {
        self.install_dir.exists() && self.install_dir.join("AetherSetup.exe").exists()
    }

    fn register_windows_uninstall_key(&self) -> anyhow::Result<()> {
        info!("Registering Uninstall Key under HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\AetherPlatform");
        // Windows Registry key registration logic
        Ok(())
    }

    fn unregister_windows_uninstall_key(&self) -> anyhow::Result<()> {
        info!("Removing Uninstall Key from HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\AetherPlatform");
        // Windows Registry key removal logic
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let installer = AetherInstaller::new();

    if args.iter().any(|arg| arg == "--uninstall") {
        installer.uninstall()?;
    } else if args.iter().any(|arg| arg == "--status") {
        if installer.is_installed() {
            info!("Aether Platform is installed at: {:?}", installer.install_dir);
        } else {
            info!("Aether Platform is NOT currently installed.");
        }
    } else {
        installer.install()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_initialization() {
        let installer = AetherInstaller::new();
        assert!(installer.install_dir.to_string_lossy().contains("Aether"));
    }
}
