use std::sync::Arc;
use ai_engine::{VoiceIntentParser, WorkflowAutomationEngine, WorkflowRule};
use async_trait::async_trait;
use ipc_protocol::ControlCommand;
use tracing::info;
use crate::event_bus::{CoreEvent, EventBus};
use crate::subsystems::{Subsystem, SubsystemHealth};

/// Core Engine Subsystem wrapping the Phase 14 AI Subsystem Engine.
pub struct AiSubsystem {
    workflow_engine: WorkflowAutomationEngine,
    event_bus: Option<Arc<EventBus>>,
}

impl AiSubsystem {
    pub fn new() -> Self {
        let mut workflow_engine = WorkflowAutomationEngine::new();
        workflow_engine.add_rule(WorkflowRule {
            rule_id: "auto_performance_theme_on_high_cpu".to_string(),
            condition_metric: "sys.cpu_usage".to_string(),
            threshold_value: 85.0,
            action_command: ControlCommand::SetThemeMode { mode: "dark".to_string() },
        });

        Self {
            workflow_engine,
            event_bus: None,
        }
    }

    pub fn parse_voice_command(&self, utterance: &str) -> Option<ControlCommand> {
        VoiceIntentParser::parse_intent(utterance)
    }
}

impl Default for AiSubsystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subsystem for AiSubsystem {
    fn name(&self) -> &'static str {
        "ai_intelligence_engine"
    }

    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing Phase 14 AI Subsystem (Desktop Automation, Voice, Layout/Theme/Widget Synthesis)...");
        self.event_bus = Some(bus);
        Ok(())
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("AiSubsystem shut down cleanly.");
        Ok(())
    }

    fn health(&self) -> SubsystemHealth {
        SubsystemHealth::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let mut subsystem = AiSubsystem::new();

        assert_eq!(subsystem.name(), "ai_intelligence_engine");
        assert!(subsystem.initialize(bus).await.is_ok());

        let cmd = subsystem.parse_voice_command("switch to dark theme").unwrap();
        assert_eq!(cmd, ControlCommand::SetThemeMode { mode: "dark".to_string() });

        assert!(subsystem.shutdown().await.is_ok());
    }
}
