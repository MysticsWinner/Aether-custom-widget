use crate::types::RecordedEvent;
use anyhow::{Context, Result};
use std::path::Path;
use tracing::info;

/// Replays recorded system events for debugging and bug reproduction.
#[derive(Debug, Clone)]
pub struct EventReplayer {
    events: Vec<RecordedEvent>,
}

impl EventReplayer {
    pub fn new(events: Vec<RecordedEvent>) -> Self {
        Self { events }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read recording file: {}", path.as_ref().display()))?;
        let events: Vec<RecordedEvent> = serde_json::from_str(&content)?;
        info!(count = events.len(), "Loaded recorded events for replayer");
        Ok(Self::new(events))
    }

    pub fn events(&self) -> &[RecordedEvent] {
        &self.events
    }

    pub fn replay_all<F>(&self, mut callback: F) -> Result<usize>
    where
        F: FnMut(&RecordedEvent) -> Result<()>,
    {
        let mut count = 0;
        for event in &self.events {
            callback(event)?;
            count += 1;
        }
        info!(count, "Completed event replay");
        Ok(count)
    }
}
