use serde::{Deserialize, Serialize};

/// Record of a single widget crash event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashRecord {
    pub widget_id: String,
    pub timestamp_ms: u64,
    pub exit_code: Option<i32>,
    pub crash_count: u32,
}

pub use crate::policy::CrashPolicy;

/// Quarantined widget metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineRecord {
    pub widget_id: String,
    pub quarantined_at_ms: u64,
    pub reason: String,
}

/// System launch mode: Normal vs Safe Mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LaunchMode {
    Normal,
    SafeMode { reason: String },
}

impl LaunchMode {
    pub fn is_safe_mode(&self) -> bool {
        matches!(self, LaunchMode::SafeMode { .. })
    }
}
