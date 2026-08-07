use serde::{Deserialize, Serialize};

/// Configurable policies for widget crash loops and exponential backoffs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrashPolicy {
    /// Maximum crashes allowed within `window_secs` before quarantining.
    pub max_crashes: u32,
    /// Rolling time window in seconds to evaluate crash count.
    pub window_secs: u64,
    /// Base duration in milliseconds for exponential backoff calculation.
    pub backoff_base_ms: u64,
}

impl Default for CrashPolicy {
    fn default() -> Self {
        Self {
            max_crashes: 5,
            window_secs: 60,
            backoff_base_ms: 1000,
        }
    }
}
