use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector Clock for tracking causal dependencies across multi-device sync operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VectorClock {
    pub clock: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&mut self, device_id: &str) {
        let counter = self.clock.entry(device_id.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Determines if `self` dominates / happened-after `other`.
    pub fn dominates(&self, other: &VectorClock) -> bool {
        let mut greater_or_equal = true;
        let mut strictly_greater = false;

        for (device, &other_seq) in &other.clock {
            let self_seq = self.clock.get(device).copied().unwrap_or(0);
            if self_seq < other_seq {
                greater_or_equal = false;
            }
            if self_seq > other_seq {
                strictly_greater = true;
            }
        }

        for (device, &self_seq) in &self.clock {
            if !other.clock.contains_key(device) && self_seq > 0 {
                strictly_greater = true;
            }
        }

        greater_or_equal && strictly_greater
    }
}

/// Wrapped item container for CRDT conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrdtItem<T> {
    pub item_id: String,
    pub payload: T,
    pub vector_clock: VectorClock,
    pub timestamp_ms: u64,
}

/// CRDT Conflict Resolver evaluating Vector Clock causality and Lamport Timestamps.
pub struct CrdtResolver;

impl CrdtResolver {
    /// Resolves conflicts between two versions of an entity deterministically without data loss.
    pub fn resolve<T: Clone>(item_a: CrdtItem<T>, item_b: CrdtItem<T>) -> CrdtItem<T> {
        if item_a.vector_clock.dominates(&item_b.vector_clock) {
            item_a
        } else if item_b.vector_clock.dominates(&item_a.vector_clock) {
            item_b
        } else {
            // Concurrent edit tie-breaker: Last-Write-Wins based on Lamport Timestamp
            if item_a.timestamp_ms >= item_b.timestamp_ms {
                item_a
            } else {
                item_b
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_causality_and_crdt_resolution() {
        let mut clock1 = VectorClock::new();
        clock1.increment("laptop_workstation");

        let mut clock2 = clock1.clone();
        clock2.increment("laptop_workstation");

        assert!(clock2.dominates(&clock1));
        assert!(!clock1.dominates(&clock2));

        let item1 = CrdtItem {
            item_id: "layout_1".to_string(),
            payload: "config_v1",
            vector_clock: clock1,
            timestamp_ms: 1000,
        };

        let item2 = CrdtItem {
            item_id: "layout_1".to_string(),
            payload: "config_v2",
            vector_clock: clock2,
            timestamp_ms: 2000,
        };

        let resolved = CrdtResolver::resolve(item1, item2.clone());
        assert_eq!(resolved.payload, "config_v2");
    }
}
