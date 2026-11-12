use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::info;

/// IT Group Policy & MDM configuration rules for Aether.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnterprisePolicy {
    pub allow_marketplace: bool,
    pub force_safe_mode: bool,
    pub require_signed_plugins: bool,
    pub blocked_widgets: HashSet<String>,
    pub allowed_capabilities: HashSet<String>,
}

impl Default for EnterprisePolicy {
    fn default() -> Self {
        let mut allowed_capabilities = HashSet::new();
        allowed_capabilities.insert("telemetry.read".to_string());
        allowed_capabilities.insert("fs.read".to_string());
        allowed_capabilities.insert("fs.write".to_string());
        allowed_capabilities.insert("network.http".to_string());

        Self {
            allow_marketplace: true,
            force_safe_mode: false,
            require_signed_plugins: true,
            blocked_widgets: HashSet::new(),
            allowed_capabilities,
        }
    }
}

/// Policy Engine evaluating active enterprise governance rules.
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    policy_path: PathBuf,
    policy: EnterprisePolicy,
}

impl PolicyEngine {
    pub fn new<P: AsRef<Path>>(policy_path: P) -> Self {
        let policy_path = policy_path.as_ref().to_path_buf();
        let policy = Self::load_from_disk(&policy_path).unwrap_or_default();
        Self { policy_path, policy }
    }

    fn load_from_disk(path: &Path) -> anyhow::Result<EnterprisePolicy> {
        if !path.exists() {
            return Ok(EnterprisePolicy::default());
        }
        let content = std::fs::read_to_string(path)?;
        let policy: EnterprisePolicy = serde_json::from_str(&content)?;
        Ok(policy)
    }

    pub fn save_to_disk(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.policy_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.policy)?;
        std::fs::write(&self.policy_path, content)?;
        Ok(())
    }

    pub fn policy(&self) -> &EnterprisePolicy {
        &self.policy
    }

    pub fn update_policy(&mut self, new_policy: EnterprisePolicy) -> anyhow::Result<()> {
        info!("Updating enterprise Group Policy rules");
        self.policy = new_policy;
        self.save_to_disk()
    }

    pub fn is_widget_allowed(&self, widget_id: &str) -> bool {
        !self.policy.blocked_widgets.contains(widget_id)
    }

    pub fn is_capability_allowed(&self, capability: &str) -> bool {
        self.policy.allowed_capabilities.contains(capability)
    }

    /// Canonical Windows Registry Group Policy path (`HKLM\SOFTWARE\Policies\Aether`).
    pub const REGISTRY_GPO_PATH: &'static str = r"HKLM\SOFTWARE\Policies\Aether";

    /// Reads live Enterprise Group Policy / MDM rules directly from the Windows Registry key
    /// `HKLM\SOFTWARE\Policies\Aether` using native Win32 `RegOpenKeyExW` / `RegQueryValueExW` APIs.
    /// If the key does not exist or access is restricted, returns the current policy unchanged.
    pub fn sync_from_windows_registry(&mut self) -> anyhow::Result<()> {
        #[cfg(windows)]
        {
            use windows::core::w;
            use windows::Win32::Foundation::WIN32_ERROR;
            use windows::Win32::System::Registry::{
                RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE,
                KEY_READ, REG_DWORD, REG_SZ, REG_VALUE_TYPE,
            };

            let mut hkey = HKEY::default();
            let subkey = w!(r"SOFTWARE\Policies\Aether");

            unsafe {
                let status = RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey, 0, KEY_READ, &mut hkey);
                if status != WIN32_ERROR(0) {
                    // Group Policy subkey not present on this machine (standard non-domain setup)
                    return Ok(());
                }

                let mut gpo_map = HashMap::new();

                let read_dword = |key: HKEY, value_name: &windows::core::PCWSTR| -> Option<u32> {
                    let mut data = 0u32;
                    let mut data_len = std::mem::size_of::<u32>() as u32;
                    let mut val_type = REG_VALUE_TYPE::default();
                    let res = RegQueryValueExW(
                        key,
                        *value_name,
                        None,
                        Some(&mut val_type),
                        Some(&mut data as *mut u32 as *mut u8),
                        Some(&mut data_len),
                    );
                    if res == WIN32_ERROR(0) && val_type == REG_DWORD {
                        Some(data)
                    } else {
                        None
                    }
                };

                let read_string = |key: HKEY, value_name: &windows::core::PCWSTR| -> Option<String> {
                    let mut buf = vec![0u16; 512];
                    let mut data_len = (buf.len() * 2) as u32;
                    let mut val_type = REG_VALUE_TYPE::default();
                    let res = RegQueryValueExW(
                        key,
                        *value_name,
                        None,
                        Some(&mut val_type),
                        Some(buf.as_mut_ptr() as *mut u8),
                        Some(&mut data_len),
                    );
                    if res == WIN32_ERROR(0) && val_type == REG_SZ {
                        let len_chars = (data_len as usize / 2).saturating_sub(1);
                        Some(String::from_utf16_lossy(&buf[..len_chars]))
                    } else {
                        None
                    }
                };

                if let Some(dw) = read_dword(hkey, &w!("AllowMarketplace")) {
                    gpo_map.insert("AllowMarketplace".to_string(), dw.to_string());
                }
                if let Some(dw) = read_dword(hkey, &w!("ForceSafeMode")) {
                    gpo_map.insert("ForceSafeMode".to_string(), dw.to_string());
                }
                if let Some(dw) = read_dword(hkey, &w!("RequireSignedPlugins")) {
                    gpo_map.insert("RequireSignedPlugins".to_string(), dw.to_string());
                }
                if let Some(s) = read_string(hkey, &w!("BlockedWidgets")) {
                    gpo_map.insert("BlockedWidgets".to_string(), s);
                }
                if let Some(s) = read_string(hkey, &w!("AllowedCapabilities")) {
                    gpo_map.insert("AllowedCapabilities".to_string(), s);
                }

                let _ = RegCloseKey(hkey);

                if !gpo_map.is_empty() {
                    info!("Successfully loaded {} GPO settings from HKLM registry", gpo_map.len());
                    return self.sync_from_ad_gpo_registry_values(&gpo_map);
                }
            }
        }

        Ok(())
    }

    /// Synchronizes Active Directory Group Policy (GPO) / MDM registry key-value pairs into the active
    /// `EnterprisePolicy` and persists the merged policy to disk.
    pub fn sync_from_ad_gpo_registry_values(
        &mut self,
        gpo_values: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let mut merged = self.policy.clone();

        if let Some(val) = gpo_values.get("AllowMarketplace") {
            merged.allow_marketplace = val == "1" || val.eq_ignore_ascii_case("true");
        }
        if let Some(val) = gpo_values.get("ForceSafeMode") {
            merged.force_safe_mode = val == "1" || val.eq_ignore_ascii_case("true");
        }
        if let Some(val) = gpo_values.get("RequireSignedPlugins") {
            merged.require_signed_plugins = val == "1" || val.eq_ignore_ascii_case("true");
        }
        if let Some(blocked_csv) = gpo_values.get("BlockedWidgets") {
            merged.blocked_widgets = blocked_csv
                .split([';', ','])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Some(allowed_caps) = gpo_values.get("AllowedCapabilities") {
            merged.allowed_capabilities = allowed_caps
                .split([';', ','])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        self.update_policy(merged)
    }
}
