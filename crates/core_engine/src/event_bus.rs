use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

/// Represents system-wide events dispatched through the Core Engine Event Bus.
#[derive(Debug, Clone, PartialEq)]
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
    /// User-defined custom event payload.
    Custom { topic: String, message: String },
}

/// Multi-threaded, lock-free Event Bus powering asynchronous event dispatching across subsystems.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<CoreEvent>,
}

impl EventBus {
    /// Creates a new `EventBus` with the specified buffer capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publishes a `CoreEvent` to all active subscribers.
    /// Returns the number of active receivers that received the event.
    pub fn publish(&self, event: CoreEvent) -> Result<usize, broadcast::error::SendError<CoreEvent>> {
        debug!(target: "event_bus", "Publishing event: {:?}", event);
        self.sender.send(event)
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
