//! Adaptive Refresh Rate Scheduler
//!
//! Provides update frequency declaration (`VeryLow`, `Low`, `Medium`, `High`, `EventDriven`)
//! to avoid unnecessary ticks and repaints.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateFrequency {
    VeryLow,    // Weather / Status (e.g. every 60s)
    Low,        // Clock / Date (e.g. every 1s)
    Medium,     // System telemetry (e.g. 100ms)
    High,       // Animations / FPS counter (e.g. 16ms / 60Hz)
    EventDriven,// Only on signal/event trigger
}

impl Default for UpdateFrequency {
    fn default() -> Self {
        UpdateFrequency::Medium
    }
}

pub struct AdaptiveRefreshScheduler;

impl AdaptiveRefreshScheduler {
    /// Returns target interval in milliseconds for a given frequency.
    pub fn target_interval_ms(freq: UpdateFrequency, is_battery_saver: bool) -> u64 {
        let base_ms = match freq {
            UpdateFrequency::VeryLow => 60_000,
            UpdateFrequency::Low => 1_000,
            UpdateFrequency::Medium => 100,
            UpdateFrequency::High => 16,
            UpdateFrequency::EventDriven => 0, // Tick strictly on event
        };

        if is_battery_saver && freq != UpdateFrequency::EventDriven {
            (base_ms * 2).max(100)
        } else {
            base_ms
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_interval_scaling() {
        assert_eq!(AdaptiveRefreshScheduler::target_interval_ms(UpdateFrequency::Low, false), 1000);
        assert_eq!(AdaptiveRefreshScheduler::target_interval_ms(UpdateFrequency::Low, true), 2000);
        assert_eq!(AdaptiveRefreshScheduler::target_interval_ms(UpdateFrequency::EventDriven, false), 0);
    }
}
