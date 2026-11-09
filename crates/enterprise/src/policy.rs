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
