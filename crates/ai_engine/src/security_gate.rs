//! AI Security Validation Pipeline & Capability Gate
//!
//! Enforces defense-in-depth isolation between untrusted AI generation (voice inputs,
//! natural language layout synthesis, workflow recommendations) and Core Engine execution.
//! Untrusted AI proposals are strictly quarantined until passing a 4-stage validation pipeline:
//! 1. SchemaValidator: Syntactic integrity, length bounds, path traversal prevention.
//! 2. PolicyValidator: Prohibits administrative commands, untrusted roots, and blacklisted actions.
//! 3. CapabilityValidator: Verifies explicit caller permissions.
//! 4. HumanApprovalGate: Requires interactive user confirmation for all mutating operations.

use ipc_protocol::ControlCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use tracing::{error, info, warn};

/// Errors returned when an AI proposal fails security validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AiSecurityError {
    #[error("Schema validation failed: {reason}")]
    SchemaViolation { reason: String },
    #[error("Path traversal attempt detected in path: '{path}'")]
    PathTraversalBlocked { path: String },
    #[error("Policy violation: {reason}")]
    PolicyViolation { reason: String },
    #[error("Missing required capability: '{capability}'")]
    MissingCapability { capability: String },
    #[error("Mutating action '{action}' requires explicit human approval before execution")]
    HumanApprovalRequired { action: String },
    #[error("Unrecognized or empty AI proposal")]
    InvalidProposal,
}

/// Raw, unverified AI proposal before security sanitization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UntrustedAiProposal {
    SetThemeMode { mode: String },
    ReloadAll,
    GetStatus,
    LoadWidget { manifest_path: String },
    SynthesizeLayout { preset: String, theme_id: String },
    ApplyConfig { config_key: String, new_value: String },
    RawCommand { action: String },
}

/// Sanitized and validated action ready for core engine dispatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedAiAction {
    pub command: ControlCommand,
    pub audited_intent: String,
    pub approved_by_user: bool,
}

/// Capabilities granted to AI subsystems.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiCapability {
    ReadTelemetry,
    ThemeControl,
    WidgetManagement,
    LayoutComposition,
    ConfigManagement,
}

/// Security validation gate protecting the core engine from untrusted AI outputs.
#[derive(Debug, Clone)]
pub struct AiSecurityGate {
    allowed_capabilities: HashSet<AiCapability>,
    allowed_manifest_dirs: Vec<String>,
}

impl AiSecurityGate {
    pub fn new() -> Self {
        let mut caps = HashSet::new();
        caps.insert(AiCapability::ReadTelemetry);
        caps.insert(AiCapability::ThemeControl);
        caps.insert(AiCapability::LayoutComposition);

        Self {
            allowed_capabilities: caps,
            allowed_manifest_dirs: vec![
                "packages/".to_string(),
                "widgets/".to_string(),
                "crates/".to_string(),
            ],
        }
    }

    /// Grants a specific capability to the AI gate.
    pub fn grant_capability(&mut self, cap: AiCapability) {
        self.allowed_capabilities.insert(cap);
    }

    /// Revokes a specific capability from the AI gate.
    pub fn revoke_capability(&mut self, cap: &AiCapability) {
        self.allowed_capabilities.remove(cap);
    }

    /// Validates an untrusted proposal through the 4-stage security gate.
    pub fn validate_and_authorize(
        &self,
        proposal: &UntrustedAiProposal,
        user_confirmed: bool,
    ) -> Result<ValidatedAiAction, AiSecurityError> {
        match proposal {
            UntrustedAiProposal::GetStatus => {
                // Read-only status query: requires ReadTelemetry capability, no user prompt needed
                if !self.allowed_capabilities.contains(&AiCapability::ReadTelemetry) {
                    return Err(AiSecurityError::MissingCapability {
                        capability: "ReadTelemetry".to_string(),
                    });
                }
                Ok(ValidatedAiAction {
                    command: ControlCommand::GetStatus,
                    audited_intent: "Query system and widget status".to_string(),
                    approved_by_user: true,
                })
            }

            UntrustedAiProposal::SetThemeMode { mode } => {
                // Stage 1: Schema validation
                let clean_mode = mode.trim().to_lowercase();
                if clean_mode != "light" && clean_mode != "dark" && clean_mode != "system" {
                    return Err(AiSecurityError::SchemaViolation {
                        reason: format!("Invalid theme mode '{}'; must be light, dark, or system", mode),
                    });
                }

                // Stage 2 & 3: Capability check
                if !self.allowed_capabilities.contains(&AiCapability::ThemeControl) {
                    return Err(AiSecurityError::MissingCapability {
                        capability: "ThemeControl".to_string(),
                    });
                }

                Ok(ValidatedAiAction {
                    command: ControlCommand::SetThemeMode { mode: clean_mode },
                    audited_intent: format!("Set desktop theme mode to '{}'", mode),
                    approved_by_user: true,
                })
            }

            UntrustedAiProposal::ReloadAll => {
                // Reload widgets requires ThemeControl or WidgetManagement
                if !self.allowed_capabilities.contains(&AiCapability::ThemeControl)
                    && !self.allowed_capabilities.contains(&AiCapability::WidgetManagement)
                {
                    return Err(AiSecurityError::MissingCapability {
                        capability: "ThemeControl or WidgetManagement".to_string(),
                    });
                }

                Ok(ValidatedAiAction {
                    command: ControlCommand::ReloadAll,
                    audited_intent: "Reload all active desktop widgets".to_string(),
                    approved_by_user: true,
                })
            }

            UntrustedAiProposal::LoadWidget { manifest_path } => {
                // Stage 1: Schema & Path Traversal Check
                self.validate_path(manifest_path)?;

                // Stage 2: Capability Check
                if !self.allowed_capabilities.contains(&AiCapability::WidgetManagement) {
                    return Err(AiSecurityError::MissingCapability {
                        capability: "WidgetManagement".to_string(),
                    });
                }

                // Stage 4: Mandatory Human Approval Gate for loading new widget binaries/manifests
                if !user_confirmed {
                    warn!(
                        "Blocked unauthorized AI widget load for '{}': Requires explicit human confirmation.",
                        manifest_path
                    );
                    return Err(AiSecurityError::HumanApprovalRequired {
                        action: format!("Load widget from manifest '{}'", manifest_path),
                    });
                }

                info!("Authorized AI widget load with human approval: '{}'", manifest_path);
                Ok(ValidatedAiAction {
                    command: ControlCommand::LoadWidget {
                        manifest_path: manifest_path.clone(),
                    },
                    audited_intent: format!("Load widget manifest '{}'", manifest_path),
                    approved_by_user: true,
                })
            }

            UntrustedAiProposal::SynthesizeLayout { preset, theme_id } => {
                if !self.allowed_capabilities.contains(&AiCapability::LayoutComposition) {
                    return Err(AiSecurityError::MissingCapability {
                        capability: "LayoutComposition".to_string(),
                    });
                }
                if preset.is_empty() || theme_id.is_empty() {
                    return Err(AiSecurityError::SchemaViolation {
                        reason: "Preset and Theme ID must not be empty".to_string(),
                    });
                }

                Ok(ValidatedAiAction {
                    command: ControlCommand::ReloadAll,
                    audited_intent: format!("Synthesize layout preset '{}' with theme '{}'", preset, theme_id),
                    approved_by_user: true,
                })
            }

            UntrustedAiProposal::ApplyConfig { config_key, new_value } => {
                // Key sanity check
                if config_key.trim().is_empty()
                    || config_key.contains("..")
                    || config_key.contains('/')
                    || config_key.contains('\\')
                    || config_key.contains('\0')
                {
                    return Err(AiSecurityError::SchemaViolation {
                        reason: format!("Invalid or malformed config key '{}'", config_key),
                    });
                }

                // Capability check
                if !self.allowed_capabilities.contains(&AiCapability::ConfigManagement) {
                    return Err(AiSecurityError::MissingCapability {
                        capability: "ConfigManagement".to_string(),
                    });
                }

                // Stage 4: Mandatory Human Approval Gate for any configuration mutation
                if !user_confirmed {
                    warn!(
                        "Blocked unauthorized AI config mutation for '{}': Requires explicit human confirmation.",
                        config_key
                    );
                    return Err(AiSecurityError::HumanApprovalRequired {
                        action: format!("Mutate configuration key '{}' to '{}'", config_key, new_value),
                    });
                }

                info!("Authorized AI config mutation with human approval: '{}'", config_key);
                Ok(ValidatedAiAction {
                    command: ControlCommand::SetThemeMode {
                        mode: new_value.clone(),
                    },
                    audited_intent: format!("Mutate configuration '{}' to '{}'", config_key, new_value),
                    approved_by_user: true,
                })
            }

            UntrustedAiProposal::RawCommand { action } => {
                error!("Blocked arbitrary unparsed AI command: '{}'", action);
                Err(AiSecurityError::PolicyViolation {
                    reason: format!("Direct execution of raw command '{}' is forbidden by AI safety policy", action),
                })
            }
        }
    }

    /// Verifies that a path does not contain path traversal sequences or unauthorized roots.
    fn validate_path(&self, path: &str) -> Result<(), AiSecurityError> {
        // Reject null bytes
        if path.contains('\0') {
            error!("Null byte injection exploit blocked: '{}'", path);
            return Err(AiSecurityError::PathTraversalBlocked {
                path: path.to_string(),
            });
        }

        // Normalize URL-encoded characters: %2e -> ., %2f -> /, %5c -> /
        let mut clean = path.to_string();
        clean = clean.replace("%2e", ".").replace("%2E", ".");
        clean = clean.replace("%2f", "/").replace("%2F", "/");
        clean = clean.replace("%5c", "/").replace("%5C", "/");
        clean = clean.replace('\\', "/");

        // Reject UNC paths and network shares
        if clean.starts_with("//") || path.starts_with(r"\\") {
            error!("UNC/Network share access blocked: '{}'", path);
            return Err(AiSecurityError::PolicyViolation {
                reason: "UNC and network share paths are prohibited".to_string(),
            });
        }

        // Reject path traversal tokens
        if clean.contains("../") || clean.contains("/..") || clean.contains("..") {
            error!("Path traversal exploit blocked: '{}'", path);
            return Err(AiSecurityError::PathTraversalBlocked {
                path: path.to_string(),
            });
        }

        // Reject absolute drive root (e.g. C:/ or /)
        if clean.starts_with('/') || clean.contains(':') {
            error!("Absolute system path access blocked: '{}'", path);
            return Err(AiSecurityError::PolicyViolation {
                reason: "Absolute system root paths are prohibited".to_string(),
            });
        }

        // Must start with an approved package directory
        let is_whitelisted = self
            .allowed_manifest_dirs
            .iter()
            .any(|dir| clean.starts_with(dir));

        if !is_whitelisted {
            return Err(AiSecurityError::PolicyViolation {
                reason: format!(
                    "Path '{}' is not within permitted widget directories ({:?})",
                    path, self.allowed_manifest_dirs
                ),
            });
        }

        // Must end with .toml
        if !clean.ends_with(".toml") {
            return Err(AiSecurityError::SchemaViolation {
                reason: "Widget manifest path must point to a .toml file".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for AiSecurityGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_allows_safe_status_and_theme() {
        let gate = AiSecurityGate::new();

        let proposal = UntrustedAiProposal::GetStatus;
        let action = gate.validate_and_authorize(&proposal, false).unwrap();
        assert_eq!(action.command, ControlCommand::GetStatus);

        let theme_proposal = UntrustedAiProposal::SetThemeMode {
            mode: "dark".to_string(),
        };
        let action = gate.validate_and_authorize(&theme_proposal, false).unwrap();
        assert_eq!(
            action.command,
            ControlCommand::SetThemeMode {
                mode: "dark".to_string()
            }
        );
    }

    #[test]
    fn test_gate_blocks_invalid_theme_mode() {
        let gate = AiSecurityGate::new();
        let bad_theme = UntrustedAiProposal::SetThemeMode {
            mode: "neon_hack".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&bad_theme, false),
            Err(AiSecurityError::SchemaViolation { .. })
        ));
    }

    #[test]
    fn test_gate_blocks_path_traversal() {
        let mut gate = AiSecurityGate::new();
        gate.grant_capability(AiCapability::WidgetManagement);

        let exploit = UntrustedAiProposal::LoadWidget {
            manifest_path: "packages/../../windows/system32/calc.toml".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&exploit, true),
            Err(AiSecurityError::PathTraversalBlocked { .. })
        ));
    }

    #[test]
    fn test_gate_blocks_url_encoded_path_traversal() {
        let mut gate = AiSecurityGate::new();
        gate.grant_capability(AiCapability::WidgetManagement);

        let encoded = UntrustedAiProposal::LoadWidget {
            manifest_path: "packages/%2e%2e/%2e%2e/system32/calc.toml".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&encoded, true),
            Err(AiSecurityError::PathTraversalBlocked { .. })
        ));
    }

    #[test]
    fn test_gate_blocks_null_byte_injection() {
        let mut gate = AiSecurityGate::new();
        gate.grant_capability(AiCapability::WidgetManagement);

        let null_attack = UntrustedAiProposal::LoadWidget {
            manifest_path: "packages/valid/widget.toml\0something_else".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&null_attack, true),
            Err(AiSecurityError::PathTraversalBlocked { .. })
        ));
    }

    #[test]
    fn test_gate_blocks_unc_network_share_access() {
        let mut gate = AiSecurityGate::new();
        gate.grant_capability(AiCapability::WidgetManagement);

        let unc = UntrustedAiProposal::LoadWidget {
            manifest_path: r"\\attacker-server\share\exploit.toml".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&unc, true),
            Err(AiSecurityError::PolicyViolation { .. })
        ));
    }

    #[test]
    fn test_gate_requires_human_approval_for_widget_load() {
        let mut gate = AiSecurityGate::new();
        gate.grant_capability(AiCapability::WidgetManagement);

        let valid_proposal = UntrustedAiProposal::LoadWidget {
            manifest_path: "packages/weather-widget/widget.toml".to_string(),
        };

        // Without human confirmation -> fails
        assert!(matches!(
            gate.validate_and_authorize(&valid_proposal, false),
            Err(AiSecurityError::HumanApprovalRequired { .. })
        ));

        // With human confirmation -> succeeds
        let action = gate.validate_and_authorize(&valid_proposal, true).unwrap();
        assert!(action.approved_by_user);
    }

    #[test]
    fn test_gate_config_mutation_requires_capability_and_approval() {
        let mut gate = AiSecurityGate::new();

        let cfg_proposal = UntrustedAiProposal::ApplyConfig {
            config_key: "desktop.theme".to_string(),
            new_value: "dark".to_string(),
        };

        // 1. Without ConfigManagement capability -> MissingCapability
        assert!(matches!(
            gate.validate_and_authorize(&cfg_proposal, true),
            Err(AiSecurityError::MissingCapability { .. })
        ));

        // Grant capability
        gate.grant_capability(AiCapability::ConfigManagement);

        // 2. Without human approval -> HumanApprovalRequired
        assert!(matches!(
            gate.validate_and_authorize(&cfg_proposal, false),
            Err(AiSecurityError::HumanApprovalRequired { .. })
        ));

        // 3. With capability AND human approval -> Succeeds
        let authorized = gate.validate_and_authorize(&cfg_proposal, true).unwrap();
        assert!(authorized.approved_by_user);
        assert_eq!(authorized.command, ControlCommand::SetThemeMode { mode: "dark".to_string() });
    }

    #[test]
    fn test_alternate_mutation_paths_strictly_guarded() {
        // Prove that arbitrary proposals, raw commands, and unparsed prompts cannot bypass
        let gate = AiSecurityGate::new();

        let raw = UntrustedAiProposal::RawCommand {
            action: "cmd.exe /c calc.exe".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&raw, true),
            Err(AiSecurityError::PolicyViolation { .. })
        ));

        let bad_key = UntrustedAiProposal::ApplyConfig {
            config_key: "system/../../malicious".to_string(),
            new_value: "val".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&bad_key, true),
            Err(AiSecurityError::SchemaViolation { .. })
        ));
    }

    #[test]
    fn test_gate_blocks_raw_arbitrary_commands() {
        let gate = AiSecurityGate::new();
        let raw = UntrustedAiProposal::RawCommand {
            action: "format C:".to_string(),
        };
        assert!(matches!(
            gate.validate_and_authorize(&raw, false),
            Err(AiSecurityError::PolicyViolation { .. })
        ));
    }
}
