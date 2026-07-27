use serde::{Deserialize, Serialize};

/// Pointer and keyboard input events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    Click { x: f32, y: f32, button: u8 },
    Hover { x: f32, y: f32 },
    KeyDown { key_code: u32 },
}

/// Unified widget event payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidgetEvent {
    Input(InputEvent),
    Telemetry { metric: String, value: f64 },
    ThemeChanged { theme_name: String },
    Custom { topic: String, payload: String },
}

/// 4. Events API Pillar Interface
pub trait EventSubscriber: Send + Sync {
    fn on_event(&mut self, event: &WidgetEvent) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEventSubscriber {
        received_count: usize,
    }

    impl EventSubscriber for MockEventSubscriber {
        fn on_event(&mut self, _event: &WidgetEvent) -> anyhow::Result<()> {
            self.received_count += 1;
            Ok(())
        }
    }

    #[test]
    fn test_event_subscriber() {
        let mut subscriber = MockEventSubscriber { received_count: 0 };
        let event = WidgetEvent::Telemetry {
            metric: "sys.cpu".to_string(),
            value: 52.4,
        };
        subscriber.on_event(&event).unwrap();
        assert_eq!(subscriber.received_count, 1);
    }
}
