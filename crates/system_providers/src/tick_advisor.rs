use serde::{Deserialize, Serialize};

/// Engine tick frequency mode based on user activity and power state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TickMode {
    /// 10ms (100 Hz) - Active user interaction
    Interactive,
    /// 50ms (20 Hz) - User idle (no input for >30s)
    Idle,
    /// 100ms (10 Hz) - On battery power or battery saver enabled
    BatterySaver,
    /// 0ms (Paused) - Fullscreen game / app active
    Paused,
}

impl TickMode {
    pub fn interval_ms(&self) -> u64 {
        match self {
            TickMode::Interactive => 10,
            TickMode::Idle => 50,
            TickMode::BatterySaver => 100,
            TickMode::Paused => 0,
        }
    }
}

/// Adaptive tick rate advisor evaluating power state, user idle time, and foreground app context.
#[derive(Debug, Clone)]
pub struct TickRateAdvisor {
    adaptive_enabled: bool,
    current_mode: TickMode,
}

impl TickRateAdvisor {
    pub fn new() -> Self {
        Self {
            adaptive_enabled: true,
            current_mode: TickMode::Interactive,
        }
    }

    pub fn is_adaptive_enabled(&self) -> bool {
        self.adaptive_enabled
    }

    pub fn set_adaptive_enabled(&mut self, enabled: bool) {
        self.adaptive_enabled = enabled;
        if !enabled {
            self.current_mode = TickMode::Interactive;
        }
    }

    pub fn current_mode(&self) -> TickMode {
        self.current_mode
    }

    pub fn evaluate(
        &mut self,
        is_on_battery: bool,
        is_fullscreen_app_active: bool,
        user_idle_secs: u64,
    ) -> TickMode {
        if !self.adaptive_enabled {
            self.current_mode = TickMode::Interactive;
            return TickMode::Interactive;
        }

        let mode = if is_fullscreen_app_active {
            TickMode::Paused
        } else if is_on_battery {
            TickMode::BatterySaver
        } else if user_idle_secs >= 30 {
            TickMode::Idle
        } else {
            TickMode::Interactive
        };

        self.current_mode = mode;
        mode
    }
}

impl Default for TickRateAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_advisor_recommends_idle_mode_on_user_inactivity() {
        let mut advisor = TickRateAdvisor::new();

        // Active state
        assert_eq!(advisor.evaluate(false, false, 5), TickMode::Interactive);
        assert_eq!(advisor.evaluate(false, false, 5).interval_ms(), 10);

        // Idle > 30s
        assert_eq!(advisor.evaluate(false, false, 35), TickMode::Idle);
        assert_eq!(advisor.evaluate(false, false, 35).interval_ms(), 50);
    }

    #[test]
    fn test_tick_advisor_battery_saver_scaling() {
        let mut advisor = TickRateAdvisor::new();

        // On battery -> BatterySaver
        assert_eq!(advisor.evaluate(true, false, 5), TickMode::BatterySaver);
        assert_eq!(advisor.evaluate(true, false, 5).interval_ms(), 100);

        // Fullscreen app -> Paused
        assert_eq!(advisor.evaluate(false, true, 5), TickMode::Paused);
        assert_eq!(advisor.evaluate(false, true, 5).interval_ms(), 0);
    }
}
