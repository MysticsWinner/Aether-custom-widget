//! AI Desktop Composer Engine
//!
//! Unifies natural language intent parsing, AI theme synthesis, layout generation,
//! material selection, and performance prediction into a cohesive pipeline guarded
//! by mandatory security capability checks and schema validation gates.

use crate::security_gate::{AiSecurityGate, UntrustedAiProposal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComposerOutput {
    pub intent_summary: String,
    pub generated_theme_id: String,
    pub layout_preset: String,
    pub recommended_material: String,
    pub predicted_cpu_pct: f32,
    pub predicted_memory_mb: f32,
    pub passes_security_gate: bool,
    pub requires_user_approval: bool,
    pub rejection_reason: Option<String>,
}

pub struct AiDesktopComposer;

impl AiDesktopComposer {
    /// Processes a natural language desktop prompt through the AI Desktop Composer pipeline.
    pub fn compose_desktop(prompt: &str) -> ComposerOutput {
        let gate = AiSecurityGate::new();
        Self::compose_desktop_with_gate(prompt, &gate)
    }

    /// Processes a prompt using a specific `AiSecurityGate` instance.
    pub fn compose_desktop_with_gate(prompt: &str, gate: &AiSecurityGate) -> ComposerOutput {
        // Sanitization & injection check
        let lower = prompt.to_lowercase();
        let has_injection = lower.contains("system32")
            || lower.contains("cmd.exe")
            || lower.contains("powershell")
            || lower.contains("format ")
            || lower.contains("../")
            || lower.contains("..\\");

        if has_injection {
            return ComposerOutput {
                intent_summary: "Rejected malicious or unsafe prompt".to_string(),
                generated_theme_id: String::new(),
                layout_preset: String::new(),
                recommended_material: String::new(),
                predicted_cpu_pct: 0.0,
                predicted_memory_mb: 0.0,
                passes_security_gate: false,
                requires_user_approval: false,
                rejection_reason: Some("Security violation: Prompt contains prohibited system tokens or traversal patterns".to_string()),
            };
        }

        let is_cyberpunk = lower.contains("cyberpunk") || lower.contains("neon");
        let is_minimal = lower.contains("minimal") || lower.contains("clean");

        let (theme_id, layout, material, cpu, mem) = if is_cyberpunk {
            ("theme.cyberpunk.neon".to_string(), "grid_3x3".to_string(), "Glass".to_string(), 0.08, 18.0)
        } else if is_minimal {
            ("theme.minimal.dark".to_string(), "single_column".to_string(), "Solid".to_string(), 0.02, 8.0)
        } else {
            ("theme.default.dark".to_string(), "flex_auto".to_string(), "Mica".to_string(), 0.05, 12.0)
        };

        let proposal = UntrustedAiProposal::SynthesizeLayout {
            preset: layout.clone(),
            theme_id: theme_id.clone(),
        };

        let (passes, reason) = match gate.validate_and_authorize(&proposal, true) {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };

        ComposerOutput {
            intent_summary: format!("Synthesized desktop setup for prompt: '{}'", prompt),
            generated_theme_id: theme_id,
            layout_preset: layout,
            recommended_material: material,
            predicted_cpu_pct: cpu,
            predicted_memory_mb: mem,
            passes_security_gate: passes,
            requires_user_approval: true, // Desktop theme / layout changes always require user confirmation
            rejection_reason: reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_desktop_composer_pipeline() {
        let output = AiDesktopComposer::compose_desktop("Make my workstation look like a cyberpunk terminal");
        assert_eq!(output.generated_theme_id, "theme.cyberpunk.neon");
        assert!(output.passes_security_gate);
        assert!(output.requires_user_approval);
        assert!(output.rejection_reason.is_none());
    }

    #[test]
    fn test_ai_desktop_composer_rejects_injection() {
        let output = AiDesktopComposer::compose_desktop("Open cmd.exe and format ../../windows");
        assert!(!output.passes_security_gate);
        assert!(output.rejection_reason.is_some());
    }
}
