use serde::{Deserialize, Serialize};

/// Recorded system event entry captured in ring buffer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordedEvent {
    pub sequence_id: u64,
    pub timestamp_ms: u64,
    pub event_type: String,
    pub payload_json: Option<String>,
}
