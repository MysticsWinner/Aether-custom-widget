use crate::types::RecordedEvent;
use anyhow::Result;
use std::collections::VecDeque;
use std::path::Path;
use tracing::info;

/// Ring-buffer recorder storing system events for playback and debugging.
#[derive(Debug, Clone)]
pub struct EventRecorder {
    events: VecDeque<RecordedEvent>,
    max_capacity: usize,
    sequence_counter: u64,
    recording_active: bool,
}

impl EventRecorder {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_capacity),
            max_capacity,
            sequence_counter: 0,
            recording_active: true,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.recording_active = active;
        info!(active, "EventRecorder active status updated");
    }

    pub fn is_active(&self) -> bool {
        self.recording_active
    }

    pub fn record(&mut self, event_type: &str, payload_json: Option<String>, now_ms: u64) -> Option<u64> {
        if !self.recording_active {
            return None;
        }

        self.sequence_counter += 1;
        let seq = self.sequence_counter;

        let entry = RecordedEvent {
            sequence_id: seq,
            timestamp_ms: now_ms,
            event_type: event_type.to_string(),
            payload_json,
        };

        if self.events.len() >= self.max_capacity {
            self.events.pop_front();
        }
        self.events.push_back(entry);

        Some(seq)
    }

    pub fn get_events(&self, from_seq: Option<u64>) -> Vec<RecordedEvent> {
        match from_seq {
            Some(seq) => self
                .events
                .iter()
                .filter(|e| e.sequence_id >= seq)
                .cloned()
                .collect(),
            None => self.events.iter().cloned().collect(),
        }
    }

    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let events = self.get_events(None);
        let content = serde_json::to_string_pretty(&events)?;
        std::fs::write(path.as_ref(), content)?;
        info!(count = events.len(), path = %path.as_ref().display(), "Exported recorded events");
        Ok(())
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}
