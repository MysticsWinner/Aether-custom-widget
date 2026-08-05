use ipc_protocol::ControlCommand;
use tracing::info;

/// Speech-to-Intent Voice Command Parser mapping spoken utterances to Core Control Commands.
pub struct VoiceIntentParser;

impl VoiceIntentParser {
    /// Translates spoken voice text into a strongly-typed `ControlCommand`.
    pub fn parse_intent(utterance: &str) -> Option<ControlCommand> {
        let text = utterance.to_lowercase();
        info!("Parsing spoken voice utterance: '{}'", utterance);

        if text.contains("dark theme") || text.contains("dark mode") {
            Some(ControlCommand::SetThemeMode { mode: "dark".to_string() })
        } else if text.contains("light theme") || text.contains("light mode") {
            Some(ControlCommand::SetThemeMode { mode: "light".to_string() })
        } else if text.contains("reload") {
            Some(ControlCommand::ReloadAll)
        } else if text.contains("ping") || text.contains("status") {
            Some(ControlCommand::GetStatus)
        } else if text.contains("load weather") {
            Some(ControlCommand::LoadWidget {
                manifest_path: "packages/weather-widget/widget.toml".to_string(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_intent_parsing() {
        let cmd1 = VoiceIntentParser::parse_intent("please switch to dark theme").unwrap();
        assert_eq!(cmd1, ControlCommand::SetThemeMode { mode: "dark".to_string() });

        let cmd2 = VoiceIntentParser::parse_intent("reload all desktop widgets").unwrap();
        assert_eq!(cmd2, ControlCommand::ReloadAll);

        let cmd3 = VoiceIntentParser::parse_intent("load weather widget").unwrap();
        assert!(matches!(cmd3, ControlCommand::LoadWidget { .. }));
    }

    #[test]
    fn test_voice_intent_light_theme() {
        let cmd = VoiceIntentParser::parse_intent("switch to light mode").unwrap();
        assert_eq!(cmd, ControlCommand::SetThemeMode { mode: "light".to_string() });
    }

    #[test]
    fn test_voice_intent_status_ping() {
        let cmd = VoiceIntentParser::parse_intent("check system status").unwrap();
        assert_eq!(cmd, ControlCommand::GetStatus);
    }
}
