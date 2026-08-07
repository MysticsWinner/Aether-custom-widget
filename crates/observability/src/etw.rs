use serde::{Deserialize, Serialize};
use tracing::info;

/// Event payload emitted via ETW provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EtwEvent {
    pub provider_name: String,
    pub event_name: String,
    pub payload_json: String,
}

/// Native Event Tracing for Windows (ETW) provider wrapper.
#[derive(Debug, Clone)]
pub struct EtwProvider {
    provider_name: String,
    enabled: bool,
}

impl EtwProvider {
    pub fn new(provider_name: &str) -> Self {
        Self {
            provider_name: provider_name.to_string(),
            enabled: true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn write_event(&self, event_name: &str, payload_json: &str) -> EtwEvent {
        if self.enabled {
            info!(
                provider = %self.provider_name,
                event = %event_name,
                "ETW Event logged"
            );
        }

        EtwEvent {
            provider_name: self.provider_name.clone(),
            event_name: event_name.to_string(),
            payload_json: payload_json.to_string(),
        }
    }
}
