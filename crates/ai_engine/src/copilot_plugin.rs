//! Windows Copilot Desktop Plugin Registration & Manifest Exporter (Phase 26)
//!
//! Exports an OpenAI / Windows Copilot compatible `ai-plugin.json` descriptor that registers
//! Aether's security-gated desktop actions (`SetThemeMode`, `SwitchProfile`, `LoadWidget`,
//! `ComposeWorkstationLayout`) with the Windows 11 Copilot runtime.

use serde::{Deserialize, Serialize};

/// Describes an action exposed by Aether to Windows Copilot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopilotActionSpec {
    pub action_id: String,
    pub description: String,
    pub requires_human_confirmation: bool,
}

/// Windows Copilot Desktop Plugin registration manifest (`ai-plugin.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopilotPluginManifest {
    pub schema_version: String,
    pub name_for_human: String,
    pub name_for_model: String,
    pub description_for_model: String,
    pub ipc_pipe_endpoint: String,
    pub actions: Vec<CopilotActionSpec>,
}

impl Default for CopilotPluginManifest {
    fn default() -> Self {
        Self {
            schema_version: "v1".to_string(),
            name_for_human: "Aether Desktop Studio".to_string(),
            name_for_model: "aether_desktop_engine".to_string(),
            description_for_model: "Controls Windows 11 desktop widgets, themes, activity profiles, and hardware telemetry overlays via the Aether AiSecurityGate.".to_string(),
            ipc_pipe_endpoint: r"\\.\pipe\CustomWidgetEngineControlPipe".to_string(),
            actions: vec![
                CopilotActionSpec {
                    action_id: "SetThemeMode".to_string(),
                    description: "Switches the desktop widget theme between dark, light, and cyberpunk tokens.".to_string(),
                    requires_human_confirmation: false,
                },
                CopilotActionSpec {
                    action_id: "SwitchActivityProfile".to_string(),
                    description: "Activates a workstation profile (Coding, Gaming, Minimalist, BatterySaver).".to_string(),
                    requires_human_confirmation: false,
                },
                CopilotActionSpec {
                    action_id: "LoadWidget".to_string(),
                    description: "Mounts a signed widget manifest onto the Windows 11 desktop layer.".to_string(),
                    requires_human_confirmation: true,
                },
            ],
        }
    }
}

impl CopilotPluginManifest {
    /// Serializes the Windows Copilot plugin registration manifest to formatted JSON.
    pub fn to_manifest_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_copilot_plugin_manifest_export() {
        let manifest = CopilotPluginManifest::default();
        assert_eq!(manifest.name_for_model, "aether_desktop_engine");
        assert_eq!(manifest.actions.len(), 3);
        assert!(manifest.actions[2].requires_human_confirmation);

        let json = manifest.to_manifest_json();
        assert!(json.contains("CustomWidgetEngineControlPipe"));
        assert!(json.contains("SwitchActivityProfile"));
    }
}

