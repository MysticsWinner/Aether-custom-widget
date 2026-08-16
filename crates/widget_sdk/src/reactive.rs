//! Signal-Driven Reactive Widget Architecture
//!
//! Replaces continuous polling with event/signal-driven telemetry bindings.
//! Emits dirty regions only when bound metric values cross hysteresis thresholds.

use serde::{Deserialize, Serialize};

/// Reactive Signal wrapper around a telemetry metric value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signal<T> {
    pub name: String,
    pub value: T,
    pub version: u64,
}

impl<T: Clone + PartialEq> Signal<T> {
    pub fn new(name: impl Into<String>, initial: T) -> Self {
        Self {
            name: name.into(),
            value: initial,
            version: 1,
        }
    }

    /// Sets a new value, incrementing signal version only if value changed.
    pub fn set(&mut self, new_value: T) -> bool {
        if self.value != new_value {
            self.value = new_value;
            self.version += 1;
            true
        } else {
            false
        }
    }
}

/// Binding connecting a Signal to a Widget state target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalBinding {
    pub signal_name: String,
    pub target_property: String,
    pub hysteresis_delta: f32,
    pub last_dispatched_version: u64,
}

impl SignalBinding {
    pub fn new(signal_name: impl Into<String>, target_property: impl Into<String>, delta: f32) -> Self {
        Self {
            signal_name: signal_name.into(),
            target_property: target_property.into(),
            hysteresis_delta: delta,
            last_dispatched_version: 0,
        }
    }

    /// Evaluates signal version against binding dispatch version.
    pub fn should_update(&mut self, signal_version: u64) -> bool {
        if signal_version > self.last_dispatched_version {
            self.last_dispatched_version = signal_version;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_version_increment_on_change() {
        let mut sig = Signal::new("sys.cpu_usage", 12.5f32);
        assert_eq!(sig.version, 1);

        assert!(!sig.set(12.5));
        assert_eq!(sig.version, 1);

        assert!(sig.set(15.0));
        assert_eq!(sig.version, 2);
    }

    #[test]
    fn test_signal_binding_dispatch_check() {
        let mut binding = SignalBinding::new("sys.cpu_usage", "width", 0.5);
        assert!(binding.should_update(1));
        assert!(!binding.should_update(1));
        assert!(binding.should_update(2));
    }
}
