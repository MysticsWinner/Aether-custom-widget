use ipc_protocol::ControlCommand;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Trigger-Condition-Action Rule for Workflow & Desktop Automation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowRule {
    pub rule_id: String,
    pub condition_metric: String,
    pub threshold_value: f64,
    pub action_command: ControlCommand,
}

/// Workflow Automation Engine evaluating rule triggers against telemetry feeds.
#[derive(Debug, Default)]
pub struct WorkflowAutomationEngine {
    rules: Vec<WorkflowRule>,
}

impl WorkflowAutomationEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: WorkflowRule) {
        info!("Registering AI Workflow Automation Rule: '{}'", rule.rule_id);
        self.rules.push(rule);
    }

    /// Evaluates telemetry value against active workflow rules.
    pub fn evaluate_telemetry(&self, metric_id: &str, value: f64) -> Vec<ControlCommand> {
        let mut triggered_actions = Vec::new();

        for rule in &self.rules {
            if rule.condition_metric == metric_id && value >= rule.threshold_value {
                info!(
                    "Workflow Rule Triggered: '{}' (Metric {} = {:.1} >= {:.1})",
                    rule.rule_id, metric_id, value, rule.threshold_value
                );
                triggered_actions.push(rule.action_command.clone());
            }
        }

        triggered_actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_automation() {
        let mut engine = WorkflowAutomationEngine::new();

        engine.add_rule(WorkflowRule {
            rule_id: "high_cpu_performance_theme".to_string(),
            condition_metric: "sys.cpu_usage".to_string(),
            threshold_value: 85.0,
            action_command: ControlCommand::SetThemeMode { mode: "dark".to_string() },
        });

        // Telemetry below threshold -> 0 triggers
        assert!(engine.evaluate_telemetry("sys.cpu_usage", 40.0).is_empty());

        // Telemetry above threshold -> Triggers action
        let actions = engine.evaluate_telemetry("sys.cpu_usage", 90.0);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], ControlCommand::SetThemeMode { mode: "dark".to_string() });
    }
}
