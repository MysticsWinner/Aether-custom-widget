//! Brendan Gregg Folded Stack FlameGraph Profiler & Exporter (Phase 21 / 25)
//!
//! Aggregates sampled call stacks (`&[&str]`) with microsecond execution durations
//! and exports standard `.folded` stack files (`frame1;frame2;frame3 <weight>`)
//! compatible with `inferno-flamegraph` and Speedscope.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Aggregated folded stack profiler for CPU & render-loop flame graph generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlameGraphProfiler {
    folded_counts: BTreeMap<String, u64>,
    total_samples: u64,
}

impl FlameGraphProfiler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a call stack sample with its weight (e.g., microseconds or sample count).
    pub fn record_stack(&mut self, frames: &[&str], weight_us: u64) {
        if frames.is_empty() || weight_us == 0 {
            return;
        }
        let folded_key = frames.join(";");
        *self.folded_counts.entry(folded_key).or_insert(0) += weight_us;
        self.total_samples += 1;
    }

    /// Exports all recorded stacks in standard Brendan Gregg `.folded` text format.
    pub fn export_folded(&self) -> String {
        let mut out = String::new();
        for (stack, weight) in &self.folded_counts {
            out.push_str(&format!("{stack} {weight}\n"));
        }
        out
    }

    /// Returns the total number of recorded stack samples.
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }

    /// Returns the number of unique collapsed call stacks.
    pub fn unique_stacks(&self) -> usize {
        self.folded_counts.len()
    }
}

