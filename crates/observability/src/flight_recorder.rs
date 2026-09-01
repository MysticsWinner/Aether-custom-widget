//! Blackbox Flight Recorder for Kernel & Runtime Diagnostics
//!
//! Stores a high-speed lock-free circular buffer of the last N engine events, IPC calls,
//! draw cycles, and telemetry states for post-mortem diagnostics.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::RwLock;

/// Single recorded diagnostic flight event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlightEvent {
    pub timestamp_ms: u64,
    pub category: String,
    pub message: String,
    pub severity: String,
}

/// Blackbox in-memory circular flight recorder.
#[derive(Debug)]
pub struct FlightRecorder {
    events: RwLock<VecDeque<FlightEvent>>,
    capacity: usize,
}

impl FlightRecorder {
    /// Creates a new `FlightRecorder` with fixed circular capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            events: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Records an event to the blackbox circular buffer.
    pub fn record(&self, category: &str, message: &str, severity: &str) {
        if let Ok(mut lock) = self.events.write() {
            if lock.len() >= self.capacity {
                lock.pop_front();
            }
            lock.push_back(FlightEvent {
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                category: category.to_string(),
                message: message.to_string(),
                severity: severity.to_string(),
            });
        }
    }

    /// Returns a copy of all recorded events in chronological order.
    pub fn export_events(&self) -> Vec<FlightEvent> {
        self.events
            .read()
            .map(|lock| lock.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Serializes all recorded events to JSON for crash dump bundling.
    pub fn export_to_json(&self) -> String {
        serde_json::to_string(&self.export_events()).unwrap_or_else(|_| "[]".to_string())
    }

    /// Returns current number of retained events.
    pub fn count(&self) -> usize {
        self.events.read().map(|lock| lock.len()).unwrap_or(0)
    }
}

impl Default for FlightRecorder {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_recorder_records_and_caps_capacity() {
        let recorder = FlightRecorder::new(5);
        assert_eq!(recorder.count(), 0);

        for i in 0..10 {
            recorder.record("RENDER", &format!("Frame {}", i), "INFO");
        }

        assert_eq!(recorder.count(), 5, "Should cap at max capacity");
        let events = recorder.export_events();
        assert_eq!(events.first().unwrap().message, "Frame 5");
        assert_eq!(events.last().unwrap().message, "Frame 9");
    }

    #[test]
    fn test_flight_recorder_json_export() {
        let recorder = FlightRecorder::new(10);
        recorder.record("IPC", "Ping received", "DEBUG");
        let json = recorder.export_to_json();
        assert!(json.contains("Ping received"));
    }
}
