pub mod recorder;
pub mod replayer;
pub mod types;

pub use recorder::EventRecorder;
pub use replayer::EventReplayer;
pub use types::RecordedEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_event_recorder_ring_buffer_capture() {
        let mut recorder = EventRecorder::new(2); // Capacity 2

        recorder.record("WidgetLoaded", Some("{\"id\":\"w1\"}".to_string()), 1000);
        recorder.record("WidgetLoaded", Some("{\"id\":\"w2\"}".to_string()), 2000);
        recorder.record("WidgetLoaded", Some("{\"id\":\"w3\"}".to_string()), 3000);

        let events = recorder.get_events(None);
        assert_eq!(events.len(), 2);
        // First event (w1) should have been popped due to ring buffer capacity
        assert_eq!(events[0].sequence_id, 2);
        assert_eq!(events[1].sequence_id, 3);
    }

    #[test]
    fn test_event_replayer_reproduces_sequence() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("recording.json");

        let mut recorder = EventRecorder::new(10);
        recorder.record("EventA", None, 100);
        recorder.record("EventB", None, 200);
        recorder.export_to_file(&file_path).unwrap();

        let replayer = EventReplayer::load_from_file(&file_path).unwrap();
        let mut replayed_types = vec![];
        replayer
            .replay_all(|ev| {
                replayed_types.push(ev.event_type.clone());
                Ok(())
            })
            .unwrap();

        assert_eq!(replayed_types, vec!["EventA", "EventB"]);
    }
}
