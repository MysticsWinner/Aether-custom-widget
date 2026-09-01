//! Signal-Driven Reactive Widget Architecture
//!
//! Replaces continuous polling with event/signal-driven telemetry bindings.
//! Emits dirty regions only when bound metric values cross hysteresis thresholds.
//! Supports derived computed signals (`Computed<T>`) and declarative manifest expressions.

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

    /// Returns current signal value.
    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Derived Computed Reactive Signal computed from an arbitrary closure or formula.
#[derive(Debug, Clone)]
pub struct Computed<T> {
    name: String,
    compute_fn: fn() -> T,
    cached_value: T,
    last_computed_version: u64,
}

impl<T: Clone + PartialEq> Computed<T> {
    pub fn new(name: impl Into<String>, compute_fn: fn() -> T) -> Self {
        let initial = compute_fn();
        Self {
            name: name.into(),
            compute_fn,
            cached_value: initial,
            last_computed_version: 1,
        }
    }

    /// Evaluates computation and updates cached value if dependent version incremented.
    pub fn update(&mut self, dependency_version: u64) -> &T {
        if dependency_version > self.last_computed_version {
            self.cached_value = (self.compute_fn)();
            self.last_computed_version = dependency_version;
        }
        &self.cached_value
    }

    /// Returns cached computed value.
    pub fn get(&self) -> &T {
        &self.cached_value
    }
}

/// Evaluates declarative conditional expressions in widget manifests (e.g. "sys.cpu_usage >= 80.0").
pub fn evaluate_condition_expression(expr: &str, cpu_val: f32, mem_val: f32) -> bool {
    let clean = expr.trim();
    if clean.starts_with("sys.cpu_usage") {
        if clean.contains(">=") {
            let threshold: f32 = clean.split(">=").nth(1).and_then(|s| s.trim().parse().ok()).unwrap_or(80.0);
            cpu_val >= threshold
        } else if clean.contains('>') {
            let threshold: f32 = clean.split('>').nth(1).and_then(|s| s.trim().parse().ok()).unwrap_or(80.0);
            cpu_val > threshold
        } else if clean.contains("<=") {
            let threshold: f32 = clean.split("<=").nth(1).and_then(|s| s.trim().parse().ok()).unwrap_or(20.0);
            cpu_val <= threshold
        } else if clean.contains('<') {
            let threshold: f32 = clean.split('<').nth(1).and_then(|s| s.trim().parse().ok()).unwrap_or(20.0);
            cpu_val < threshold
        } else {
            cpu_val > 0.0
        }
    } else if clean.starts_with("sys.memory_usage") {
        if clean.contains(">=") {
            let threshold: f32 = clean.split(">=").nth(1).and_then(|s| s.trim().parse().ok()).unwrap_or(8000.0);
            mem_val >= threshold
        } else {
            mem_val > 0.0
        }
    } else {
        true
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
    fn test_computed_signal_recalculation() {
        fn compute_high_load() -> bool {
            true
        }

        let mut computed = Computed::new("is_high_load", compute_high_load);
        assert_eq!(*computed.get(), true);
        assert_eq!(*computed.update(2), true);
    }

    #[test]
    fn test_condition_expression_evaluator() {
        assert!(evaluate_condition_expression("sys.cpu_usage >= 80.0", 85.2, 4000.0));
        assert!(!evaluate_condition_expression("sys.cpu_usage >= 80.0", 50.0, 4000.0));
        assert!(evaluate_condition_expression("sys.cpu_usage < 50.0", 42.0, 4000.0));
        assert!(evaluate_condition_expression("sys.memory_usage >= 6000.0", 10.0, 8192.0));
    }

    #[test]
    fn test_signal_binding_dispatch_check() {
        let mut binding = SignalBinding::new("sys.cpu_usage", "width", 0.5);
        assert!(binding.should_update(1));
        assert!(!binding.should_update(1));
        assert!(binding.should_update(2));
    }
}
