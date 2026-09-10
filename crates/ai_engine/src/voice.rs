use crate::security_gate::{AiSecurityError, AiSecurityGate, UntrustedAiProposal, ValidatedAiAction};
use tracing::info;

/// Speech-to-Intent Voice Command Parser mapping spoken utterances to Untrusted AI Proposals.
/// Direct generation of executable ControlCommand without security gate validation is strictly forbidden.
pub struct VoiceIntentParser;

impl VoiceIntentParser {
    /// Translates spoken voice text into an `UntrustedAiProposal`.
    pub fn parse_intent(utterance: &str) -> Option<UntrustedAiProposal> {
        let text = utterance.to_lowercase();
        info!("Parsing spoken voice utterance into untrusted proposal: '{}'", utterance);

        if text.contains("dark theme") || text.contains("dark mode") {
            Some(UntrustedAiProposal::SetThemeMode { mode: "dark".to_string() })
        } else if text.contains("light theme") || text.contains("light mode") {
            Some(UntrustedAiProposal::SetThemeMode { mode: "light".to_string() })
        } else if text.contains("reload") {
            Some(UntrustedAiProposal::ReloadAll)
        } else if text.contains("ping") || text.contains("status") {
            Some(UntrustedAiProposal::GetStatus)
        } else if text.contains("load weather") {
            Some(UntrustedAiProposal::LoadWidget {
                manifest_path: "packages/weather-widget/widget.toml".to_string(),
            })
        } else {
            None
        }
    }

    /// Parses utterance into an untrusted proposal and runs it through the security gate.
    pub fn parse_and_authorize(
        utterance: &str,
        gate: &AiSecurityGate,
        user_confirmed: bool,
    ) -> Result<ValidatedAiAction, AiSecurityError> {
        let proposal = Self::parse_intent(utterance).ok_or(AiSecurityError::InvalidProposal)?;
        gate.validate_and_authorize(&proposal, user_confirmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_gate::AiCapability;
    use ipc_protocol::ControlCommand;

    #[test]
    fn test_voice_intent_parsing_generates_untrusted_proposals() {
        let p1 = VoiceIntentParser::parse_intent("please switch to dark theme").unwrap();
        assert_eq!(p1, UntrustedAiProposal::SetThemeMode { mode: "dark".to_string() });

        let p2 = VoiceIntentParser::parse_intent("reload all desktop widgets").unwrap();
        assert_eq!(p2, UntrustedAiProposal::ReloadAll);

        let p3 = VoiceIntentParser::parse_intent("load weather widget").unwrap();
        assert!(matches!(p3, UntrustedAiProposal::LoadWidget { .. }));
    }

    #[test]
    fn test_voice_intent_end_to_end_validation() {
        let mut gate = AiSecurityGate::new();
        gate.grant_capability(AiCapability::WidgetManagement);

        // Status check passes without confirmation
        let action1 = VoiceIntentParser::parse_and_authorize("check system status", &gate, false).unwrap();
        assert_eq!(action1.command, ControlCommand::GetStatus);

        // Theme switch passes without confirmation
        let action2 = VoiceIntentParser::parse_and_authorize("switch to dark mode", &gate, false).unwrap();
        assert_eq!(action2.command, ControlCommand::SetThemeMode { mode: "dark".to_string() });

        // Widget load requires confirmation
        let fail = VoiceIntentParser::parse_and_authorize("load weather widget", &gate, false);
        assert!(matches!(fail, Err(AiSecurityError::HumanApprovalRequired { .. })));

        let pass = VoiceIntentParser::parse_and_authorize("load weather widget", &gate, true).unwrap();
        assert!(matches!(pass.command, ControlCommand::LoadWidget { .. }));
    }
}
