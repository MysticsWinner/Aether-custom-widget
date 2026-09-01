//! Context-Aware Desktop Automation Engine
//!
//! Monitors system context signals (foreground application, running process, fullscreen mode,
//! power state, battery state, display topology) and triggers atomic profile switching with rollback.

use serde::{Deserialize, Serialize};

/// System context signals captured from the OS environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextSignal {
    pub foreground_app: Option<String>,
    pub is_fullscreen: bool,
    pub is_battery_saver: bool,
    pub active_display_count: u32,
    pub is_gaming_process: bool,
}

impl Default for ContextSignal {
    fn default() -> Self {
        Self {
            foreground_app: None,
            is_fullscreen: false,
            is_battery_saver: false,
            active_display_count: 1,
            is_gaming_process: false,
        }
    }
}

/// Dynamic condition trigger rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerCondition {
    Fullscreen3D,
    BatterySaver,
    ProcessMatch(String),
    MultiDisplay(u32),
}

/// Context trigger rule mapping condition to target profile ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextRule {
    pub name: String,
    pub condition: TriggerCondition,
    pub target_profile_id: String,
    pub priority: u32,
}

/// Autonomous Context-Aware Engine.
#[derive(Debug, Clone)]
pub struct ContextAwareEngine {
    rules: Vec<ContextRule>,
    active_profile_id: String,
    profile_history: Vec<String>,
}

impl ContextAwareEngine {
    pub fn new() -> Self {
        Self {
            rules: vec![
                ContextRule {
                    name: "Fullscreen Gaming".to_string(),
                    condition: TriggerCondition::Fullscreen3D,
                    target_profile_id: "profile.gaming".to_string(),
                    priority: 100,
                },
                ContextRule {
                    name: "Battery Saver".to_string(),
                    condition: TriggerCondition::BatterySaver,
                    target_profile_id: "profile.travel".to_string(),
                    priority: 90,
                },
                ContextRule {
                    name: "VS Code / Visual Studio".to_string(),
                    condition: TriggerCondition::ProcessMatch("code".to_string()),
                    target_profile_id: "profile.coding".to_string(),
                    priority: 50,
                },
                ContextRule {
                    name: "Multi-Monitor Workspace".to_string(),
                    condition: TriggerCondition::MultiDisplay(2),
                    target_profile_id: "profile.work".to_string(),
                    priority: 30,
                },
            ],
            active_profile_id: "profile.default".to_string(),
            profile_history: Vec::new(),
        }
    }

    /// Evaluates dynamic context signal to determine target desktop profile ID.
    pub fn evaluate_context_target(&self, signal: &ContextSignal) -> Option<String> {
        let mut matching_rules: Vec<&ContextRule> = self
            .rules
            .iter()
            .filter(|rule| match &rule.condition {
                TriggerCondition::Fullscreen3D => signal.is_fullscreen || signal.is_gaming_process,
                TriggerCondition::BatterySaver => signal.is_battery_saver,
                TriggerCondition::ProcessMatch(proc_name) => {
                    if let Some(app) = &signal.foreground_app {
                        app.to_lowercase().contains(&proc_name.to_lowercase())
                    } else {
                        false
                    }
                }
                TriggerCondition::MultiDisplay(min_count) => signal.active_display_count >= *min_count,
            })
            .collect();

        // Sort by priority descending
        matching_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        matching_rules.first().map(|r| r.target_profile_id.clone())
    }

    /// Updates context state and transitions profile if target changed.
    pub fn update_context(&mut self, signal: &ContextSignal) -> Option<String> {
        if let Some(target) = self.evaluate_context_target(signal) {
            if target != self.active_profile_id {
                self.profile_history.push(self.active_profile_id.clone());
                self.active_profile_id = target.clone();
                return Some(target);
            }
        }
        None
    }

    /// Reverts to the previous profile in history stack.
    pub fn rollback_profile(&mut self) -> Option<String> {
        if let Some(prev) = self.profile_history.pop() {
            self.active_profile_id = prev.clone();
            Some(prev)
        } else {
            None
        }
    }

    /// Returns currently active profile ID.
    pub fn active_profile_id(&self) -> &str {
        &self.active_profile_id
    }
}

impl Default for ContextAwareEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_aware_engine_fullscreen_trigger() {
        let engine = ContextAwareEngine::new();
        let signal = ContextSignal {
            is_fullscreen: true,
            ..Default::default()
        };

        let target = engine.evaluate_context_target(&signal);
        assert_eq!(target, Some("profile.gaming".to_string()));
    }

    #[test]
    fn test_context_aware_engine_coding_trigger() {
        let engine = ContextAwareEngine::new();
        let signal = ContextSignal {
            foreground_app: Some("Code.exe - Aether".to_string()),
            ..Default::default()
        };

        let target = engine.evaluate_context_target(&signal);
        assert_eq!(target, Some("profile.coding".to_string()));
    }

    #[test]
    fn test_context_aware_engine_transition_and_rollback() {
        let mut engine = ContextAwareEngine::new();
        assert_eq!(engine.active_profile_id(), "profile.default");

        let signal = ContextSignal {
            is_battery_saver: true,
            ..Default::default()
        };

        let switched = engine.update_context(&signal);
        assert_eq!(switched, Some("profile.travel".to_string()));
        assert_eq!(engine.active_profile_id(), "profile.travel");

        let restored = engine.rollback_profile();
        assert_eq!(restored, Some("profile.default".to_string()));
        assert_eq!(engine.active_profile_id(), "profile.default");
    }
}
