//! Context-Aware Desktop Automation Engine
//!
//! Monitors system context signals (foreground application, running process, fullscreen mode,
//! power state, battery state, display topology) and triggers atomic profile switching.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextSignal {
    pub foreground_app: Option<String>,
    pub is_fullscreen: bool,
    pub is_battery_saver: bool,
    pub active_display_count: u32,
}

impl Default for ContextSignal {
    fn default() -> Self {
        Self {
            foreground_app: None,
            is_fullscreen: false,
            is_battery_saver: false,
            active_display_count: 1,
        }
    }
}

pub struct ContextAwareEngine;

impl ContextAwareEngine {
    /// Evaluates dynamic context signal to determine target desktop profile ID.
    pub fn evaluate_context_target(signal: &ContextSignal) -> Option<String> {
        if signal.is_fullscreen {
            return Some("profile.gaming".to_string());
        }

        if signal.is_battery_saver {
            return Some("profile.travel".to_string());
        }

        if let Some(app) = &signal.foreground_app {
            if app.to_lowercase().contains("devenv") || app.to_lowercase().contains("code") {
                return Some("profile.coding".to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_aware_engine_fullscreen_trigger() {
        let signal = ContextSignal {
            is_fullscreen: true,
            ..Default::default()
        };

        let target = ContextAwareEngine::evaluate_context_target(&signal);
        assert_eq!(target, Some("profile.gaming".to_string()));
    }
}
