use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, trace, warn};

/// Classification of event reliability and delivery semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventReliability {
    /// High-frequency lossy/ephemeral data (e.g. TelemetryTick, FrameStats) where dropping on lag is acceptable.
    Ephemeral,
    /// In-memory replayable state transitions (e.g. SystemStateChanged, ThemeChanged, WidgetLifecycle, ProfileChanged)
    /// preserved in the bounded circular sequence replay buffer for reconnection recovery.
    Replayable,
    /// Persisted durable operations (e.g. atomic configuration transactions, WAL logs, snapshot exports).
    Durable,
}

/// Represents system-wide events dispatched through the Core Engine Event Bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoreEvent {
    /// System state event (e.g. startup, shutdown, pause, resume).
    SystemStateChanged { state: String },
    /// Theme change notification (e.g. light, dark, custom theme token update).
    ThemeChanged { theme_name: String },
    /// Hardware telemetry tick containing metric identifier and value.
    TelemetryTick { metric_id: String, value: f64 },
    /// Signal dispatched to or received from a subsystem module.
    SubsystemSignal { subsystem: String, signal: String },
    /// Command sent via IPC or dashboard control.
    ControlCommand { command: String, payload: String },
    /// Widget lifecycle event (e.g. loaded, unloaded, mounted, unmounted).
    WidgetLifecycle { widget_id: String, state: String },
    /// Profile switch notification.
    ProfileChanged { profile_name: String },
    /// User-defined custom event payload.
    Custom { topic: String, message: String },
}

impl CoreEvent {
    /// Returns the delivery and reliability classification of this event.
    pub fn reliability(&self) -> EventReliability {
        match self {
            CoreEvent::TelemetryTick { .. } => EventReliability::Ephemeral,
            CoreEvent::ControlCommand { .. } => EventReliability::Durable,
            _ => EventReliability::Replayable,
        }
    }
}

/// A timestamped, sequence-numbered event stored in the replay buffer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequencedEvent {
    pub sequence: u64,
    pub reliability: EventReliability,
    pub event: CoreEvent,
}

/// Authoritative system state snapshot used for full reconciliation when replay gaps occur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoritativeStateSnapshot {
    pub snapshot_sequence: u64,
    pub system_state: String,
    pub current_theme: Option<String>,
    pub current_profile: Option<String>,
}

/// Result of querying the replay buffer for events since a given sequence number.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayResult {
    /// Continuous history: all events since `since_sequence` were retained in the buffer.
    Continuous(Vec<SequencedEvent>),
    /// Gap detected: the requested sequence was pruned from the circular buffer.
    /// Returns the gap boundaries, an authoritative state snapshot, and all remaining available events.
    GapDetected {
        requested_sequence: u64,
        oldest_available_sequence: u64,
        authoritative_snapshot: AuthoritativeStateSnapshot,
        events: Vec<SequencedEvent>,
    },
}

/// Asynchronous Event Bus powering inter-subsystem communication.
///
/// Architecture Guarantee:
/// - Broadcast channels provide low-latency delivery for active subscribers.
/// - Replayable and Durable state events are sequence-numbered and preserved in a bounded circular `replay_buffer`.
/// - If a subscriber experiences a sequence gap (`since_sequence < oldest_available`), the bus explicitly returns
///   `ReplayResult::GapDetected` paired with an authoritative state snapshot to force reconciliation.
/// - Invariant: Dropped ephemeral events NEVER corrupt authoritative system state.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<CoreEvent>,
    sequence: Arc<AtomicU64>,
    replay_buffer: Arc<RwLock<VecDeque<SequencedEvent>>>,
    replay_capacity: usize,
    system_state: Arc<RwLock<String>>,
    current_theme: Arc<RwLock<Option<String>>>,
    current_profile: Arc<RwLock<Option<String>>>,
}

impl EventBus {
    /// Creates a new `EventBus` with the specified broadcast capacity and default replay buffer (128 events).
    pub fn new(capacity: usize) -> Self {
        Self::with_replay_capacity(capacity, 128)
    }

    /// Creates a new `EventBus` with specified broadcast channel capacity and replay buffer size.
    pub fn with_replay_capacity(capacity: usize, replay_capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            sequence: Arc::new(AtomicU64::new(0)),
            replay_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(replay_capacity))),
            replay_capacity,
            system_state: Arc::new(RwLock::new("Uninitialized".to_string())),
            current_theme: Arc::new(RwLock::new(None)),
            current_profile: Arc::new(RwLock::new(None)),
        }
    }

    /// Publishes a `CoreEvent` to all active subscribers.
    /// If the event is Replayable or Durable, it is sequence-numbered, committed to the replay buffer,
    /// and used to update authoritative state caches before broadcasting.
    pub fn publish(&self, event: CoreEvent) -> Result<usize, broadcast::error::SendError<CoreEvent>> {
        debug!(target: "event_bus", "Publishing event: {:?}", event);

        // Update authoritative caches synchronously for state transitions
        match &event {
            CoreEvent::SystemStateChanged { state } => {
                if let Ok(mut lock) = self.system_state.try_write() {
                    *lock = state.clone();
                }
            }
            CoreEvent::ThemeChanged { theme_name } => {
                if let Ok(mut lock) = self.current_theme.try_write() {
                    *lock = Some(theme_name.clone());
                }
            }
            CoreEvent::ProfileChanged { profile_name } => {
                if let Ok(mut lock) = self.current_profile.try_write() {
                    *lock = Some(profile_name.clone());
                }
            }
            _ => {}
        }

        // If event is Replayable or Durable, record into replay buffer with sequence
        let rel = event.reliability();
        if rel == EventReliability::Replayable || rel == EventReliability::Durable {
            let seq = self.sequence.fetch_add(1, Ordering::SeqCst) + 1;
            let sequenced = SequencedEvent {
                sequence: seq,
                reliability: rel,
                event: event.clone(),
            };

            if let Ok(mut buf) = self.replay_buffer.try_write() {
                if buf.len() >= self.replay_capacity {
                    buf.pop_front();
                }
                buf.push_back(sequenced);
            }
        }

        match self.sender.send(event) {
            Ok(receiver_count) => {
                trace!(target: "event_bus", receiver_count, "Event broadcast successfully");
                Ok(receiver_count)
            }
            Err(e) => {
                trace!(target: "event_bus", "Event broadcast completed with 0 active receivers");
                Err(e)
            }
        }
    }

    /// Replays events dispatched since `since_sequence`.
    /// If `since_sequence` has been pruned due to buffer capacity overflow, returns `ReplayResult::GapDetected`
    /// containing an authoritative state snapshot so the client can reconcile state deterministically.
    pub async fn replay_since(&self, since_sequence: u64) -> ReplayResult {
        let buf = self.replay_buffer.read().await;
        if buf.is_empty() {
            return ReplayResult::Continuous(Vec::new());
        }

        let oldest_available = buf.front().map(|e| e.sequence).unwrap_or(0);
        let events: Vec<SequencedEvent> = buf
            .iter()
            .filter(|e| e.sequence > since_sequence)
            .cloned()
            .collect();

        // Check if there is a gap between requested sequence and oldest available
        if since_sequence > 0 && since_sequence + 1 < oldest_available {
            warn!(
                target: "event_bus",
                requested = since_sequence,
                oldest_available,
                "Event replay gap detected — forcing state reconciliation with authoritative snapshot"
            );
            let snapshot = self.get_authoritative_snapshot().await;
            ReplayResult::GapDetected {
                requested_sequence: since_sequence,
                oldest_available_sequence: oldest_available,
                authoritative_snapshot: snapshot,
                events,
            }
        } else {
            trace!(target: "event_bus", requested = since_sequence, count = events.len(), "Continuous event replay served");
            ReplayResult::Continuous(events)
        }
    }

    /// Returns the vector of replayed events directly (unwrapped).
    pub async fn replay_since_events(&self, since_sequence: u64) -> Vec<SequencedEvent> {
        match self.replay_since(since_sequence).await {
            ReplayResult::Continuous(events) => events,
            ReplayResult::GapDetected { events, .. } => events,
        }
    }

    /// Returns the current sequence number for reliable events.
    pub fn current_sequence(&self) -> u64 {
        self.sequence.load(Ordering::Acquire)
    }

    /// Captures a complete authoritative state snapshot for client reconciliation.
    pub async fn get_authoritative_snapshot(&self) -> AuthoritativeStateSnapshot {
        AuthoritativeStateSnapshot {
            snapshot_sequence: self.current_sequence(),
            system_state: self.system_state.read().await.clone(),
            current_theme: self.current_theme.read().await.clone(),
            current_profile: self.current_profile.read().await.clone(),
        }
    }

    /// Queries the authoritative current system state.
    pub async fn current_system_state(&self) -> String {
        self.system_state.read().await.clone()
    }

    /// Queries the authoritative current theme name.
    pub async fn current_theme(&self) -> Option<String> {
        self.current_theme.read().await.clone()
    }

    /// Queries the authoritative current desktop profile name.
    pub async fn current_profile(&self) -> Option<String> {
        self.current_profile.read().await.clone()
    }

    /// Creates a new subscriber receiver channel for listening to events.
    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.sender.subscribe()
    }

    /// Returns the current number of active subscribers.
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Dedicated publisher handle for emitting events into the Event Bus.
#[derive(Clone)]
pub struct EventPublisher {
    bus: Arc<EventBus>,
}

impl EventPublisher {
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    pub fn emit(&self, event: CoreEvent) -> Result<usize, broadcast::error::SendError<CoreEvent>> {
        self.bus.publish(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        assert_eq!(bus.receiver_count(), 2);

        let test_event = CoreEvent::SystemStateChanged {
            state: "Running".to_string(),
        };

        let send_count = bus.publish(test_event.clone()).unwrap();
        assert_eq!(send_count, 2);

        let received1 = timeout(Duration::from_millis(100), rx1.recv())
            .await
            .unwrap()
            .unwrap();
        let received2 = timeout(Duration::from_millis(100), rx2.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(received1, test_event);
        assert_eq!(received2, test_event);
    }

    #[tokio::test]
    async fn test_reliable_event_replay_and_state_recovery() {
        let bus = EventBus::with_replay_capacity(16, 32);
        let _rx = bus.subscribe();

        // 1. Publish replayable events
        bus.publish(CoreEvent::SystemStateChanged { state: "Running".to_string() }).unwrap();
        bus.publish(CoreEvent::ThemeChanged { theme_name: "cyberpunk_neon".to_string() }).unwrap();
        bus.publish(CoreEvent::ProfileChanged { profile_name: "Gaming".to_string() }).unwrap();

        // 2. Publish ephemeral event (should NOT be added to replay buffer)
        bus.publish(CoreEvent::TelemetryTick { metric_id: "sys.cpu".to_string(), value: 45.0 }).unwrap();

        assert_eq!(bus.current_sequence(), 3);
        assert_eq!(bus.current_system_state().await, "Running");
        assert_eq!(bus.current_theme().await, Some("cyberpunk_neon".to_string()));
        assert_eq!(bus.current_profile().await, Some("Gaming".to_string()));

        // 3. Replay from sequence 1 (should return events 2 and 3 continuously)
        match bus.replay_since(1).await {
            ReplayResult::Continuous(replayed) => {
                assert_eq!(replayed.len(), 2);
                assert_eq!(replayed[0].sequence, 2);
                assert_eq!(replayed[0].event, CoreEvent::ThemeChanged { theme_name: "cyberpunk_neon".to_string() });
                assert_eq!(replayed[1].sequence, 3);
                assert_eq!(replayed[1].event, CoreEvent::ProfileChanged { profile_name: "Gaming".to_string() });
            }
            ReplayResult::GapDetected { .. } => panic!("Expected continuous replay"),
        }
    }

    #[tokio::test]
    async fn test_replay_buffer_gap_detection_forces_snapshot_reconciliation() {
        // Buffer capacity = 3
        let bus = EventBus::with_replay_capacity(16, 3);
        let _rx = bus.subscribe();

        // Publish 5 events: buffer will retain only events 3, 4, 5 (events 1 & 2 evicted)
        for i in 1..=5 {
            bus.publish(CoreEvent::ThemeChanged {
                theme_name: format!("theme_{}", i),
            }).unwrap();
        }

        assert_eq!(bus.current_sequence(), 5);

        // Subscriber asks for replay since sequence 1 (gap: events 2 was evicted, oldest is 3)
        let result = bus.replay_since(1).await;
        match result {
            ReplayResult::GapDetected {
                requested_sequence,
                oldest_available_sequence,
                authoritative_snapshot,
                events,
            } => {
                assert_eq!(requested_sequence, 1);
                assert_eq!(oldest_available_sequence, 3);
                assert_eq!(authoritative_snapshot.current_theme, Some("theme_5".to_string()));
                assert_eq!(authoritative_snapshot.snapshot_sequence, 5);
                assert_eq!(events.len(), 3); // events 3, 4, 5
            }
            ReplayResult::Continuous(_) => panic!("Expected GapDetected due to eviction"),
        }
    }

    #[tokio::test]
    async fn test_multi_threaded_event_dispatch() {
        let bus = Arc::new(EventBus::new(32));
        let publisher = EventPublisher::new(bus.clone());

        let mut rx = bus.subscribe();

        let handle = tokio::spawn(async move {
            publisher
                .emit(CoreEvent::TelemetryTick {
                    metric_id: "cpu_usage".to_string(),
                    value: 42.5,
                })
                .unwrap();
        });

        handle.await.unwrap();

        let event = timeout(Duration::from_millis(100), rx.recv())
            .await
            .unwrap()
            .unwrap();

        if let CoreEvent::TelemetryTick { metric_id, value } = event {
            assert_eq!(metric_id, "cpu_usage");
            assert_eq!(value, 42.5);
        } else {
            panic!("Expected TelemetryTick event");
        }
    }
}
