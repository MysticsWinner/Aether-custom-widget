use crate::rendering::RectF;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// Root cause explaining why a display region was marked dirty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidationCause {
    /// Incoming telemetry metric updated.
    MetricChanged { metric_id: String },
    /// Active desktop theme or glassmorphism token changed.
    ThemeChanged,
    /// Continuous animation or physics tick advanced.
    AnimationTick { frame: u64 },
    /// Dynamic flexbox layout or widget resize occurred.
    LayoutResize,
    /// Widget visibility toggled or virtual desktop switched.
    VisibilityToggled { visible: bool },
    /// User input or mouse interaction.
    UserInteraction,
    /// Explicit full or partial redraw request.
    ExplicitForceRedraw,
}

impl InvalidationCause {
    pub fn category_name(&self) -> &'static str {
        match self {
            InvalidationCause::MetricChanged { .. } => "MetricChanged",
            InvalidationCause::ThemeChanged => "ThemeChanged",
            InvalidationCause::AnimationTick { .. } => "AnimationTick",
            InvalidationCause::LayoutResize => "LayoutResize",
            InvalidationCause::VisibilityToggled { .. } => "VisibilityToggled",
            InvalidationCause::UserInteraction => "UserInteraction",
            InvalidationCause::ExplicitForceRedraw => "ExplicitForceRedraw",
        }
    }
}

/// An invalidated screen rectangle paired with its invalidation cause and timestamp.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvalidatedRegion {
    pub rect: RectF,
    pub cause: InvalidationCause,
    pub timestamp_ms: u64,
}

/// Tracks invalidated screen bounds to enforce dirty rectangle partial rendering.
/// Ensures zero unnecessary redraws when screen contents are static.
#[derive(Debug, Clone)]
pub struct DirtyRegionTracker {
    dirty_regions: Vec<InvalidatedRegion>,
    max_regions: usize,
    cause_counts: HashMap<String, u64>,
}

impl DirtyRegionTracker {
    /// Creates a new `DirtyRegionTracker` with a maximum tracking capacity.
    pub fn new(max_regions: usize) -> Self {
        Self {
            dirty_regions: Vec::with_capacity(max_regions),
            max_regions,
            cause_counts: HashMap::new(),
        }
    }

    /// Adds a region to be invalidated for partial redraw with an explicit root cause.
    pub fn add_region_with_cause(&mut self, rect: RectF, cause: InvalidationCause) {
        if rect.is_empty() {
            return;
        }

        let cat = cause.category_name().to_string();
        *self.cause_counts.entry(cat).or_insert(0) += 1;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        debug!(
            "Invalidating dirty region: ({:.1}, {:.1}, {:.1}x{:.1}) [cause: {:?}]",
            rect.x, rect.y, rect.width, rect.height, cause
        );

        // Merge with existing intersecting regions if possible to reduce draw calls
        for region in self.dirty_regions.iter_mut() {
            if region.rect.intersects(&rect) {
                region.rect = region.rect.union(&rect);
                region.cause = cause;
                region.timestamp_ms = now;
                return;
            }
        }

        if self.dirty_regions.len() >= self.max_regions {
            // If capacity limit reached, collapse all into a single consolidated bounding box
            let combined = self.bounding_box();
            self.dirty_regions.clear();
            self.dirty_regions.push(InvalidatedRegion {
                rect: combined.union(&rect),
                cause,
                timestamp_ms: now,
            });
        } else {
            self.dirty_regions.push(InvalidatedRegion {
                rect,
                cause,
                timestamp_ms: now,
            });
        }
    }

    /// Adds a region with default `ExplicitForceRedraw` cause (backward-compatible).
    pub fn add_region(&mut self, rect: RectF) {
        self.add_region_with_cause(rect, InvalidationCause::ExplicitForceRedraw);
    }

    /// Returns whether any dirty regions are queued for rendering.
    pub fn is_dirty(&self) -> bool {
        !self.dirty_regions.is_empty()
    }

    /// Returns a slice of all active invalidated regions.
    pub fn regions(&self) -> &[InvalidatedRegion] {
        &self.dirty_regions
    }

    /// Returns a vector of raw bounding rectangles for the dirty regions.
    pub fn rects(&self) -> Vec<RectF> {
        self.dirty_regions.iter().map(|r| r.rect).collect()
    }

    /// Computes the single bounding rectangle containing all dirty regions.
    pub fn bounding_box(&self) -> RectF {
        if self.dirty_regions.is_empty() {
            return RectF::zero();
        }
        let mut bbox = self.dirty_regions[0].rect;
        for region in &self.dirty_regions[1..] {
            bbox = bbox.union(&region.rect);
        }
        bbox
    }

    /// Clears all dirty regions after a successful render frame pass.
    pub fn clear(&mut self) {
        self.dirty_regions.clear();
    }

    /// Returns statistics on invalidation triggers.
    pub fn cause_stats(&self) -> &HashMap<String, u64> {
        &self.cause_counts
    }
}

impl Default for DirtyRegionTracker {
    fn default() -> Self {
        Self::new(32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_tracker_zero_redraw() {
        let tracker = DirtyRegionTracker::default();
        assert!(!tracker.is_dirty());
        assert_eq!(tracker.regions().len(), 0);
    }

    #[test]
    fn test_dirty_tracker_merge_overlapping() {
        let mut tracker = DirtyRegionTracker::default();
        tracker.add_region_with_cause(
            RectF::new(10.0, 10.0, 50.0, 50.0),
            InvalidationCause::MetricChanged {
                metric_id: "sys.cpu".into(),
            },
        );
        tracker.add_region_with_cause(
            RectF::new(30.0, 30.0, 50.0, 50.0),
            InvalidationCause::AnimationTick { frame: 12 },
        );

        assert!(tracker.is_dirty());
        assert_eq!(tracker.regions().len(), 1);
        assert_eq!(tracker.bounding_box(), RectF::new(10.0, 10.0, 70.0, 70.0));
        assert_eq!(tracker.cause_stats().get("MetricChanged"), Some(&1));
        assert_eq!(tracker.cause_stats().get("AnimationTick"), Some(&1));
    }

    #[test]
    fn test_dirty_tracker_disjoint_regions() {
        let mut tracker = DirtyRegionTracker::default();
        tracker.add_region(RectF::new(0.0, 0.0, 10.0, 10.0));
        tracker.add_region(RectF::new(100.0, 100.0, 10.0, 10.0));

        assert_eq!(tracker.regions().len(), 2);
        tracker.clear();
        assert!(!tracker.is_dirty());
    }

    #[test]
    fn test_dirty_tracker_capacity_bounding_box_collapse() {
        let mut tracker = DirtyRegionTracker::new(2);
        tracker.add_region(RectF::new(0.0, 0.0, 10.0, 10.0));
        tracker.add_region(RectF::new(20.0, 20.0, 10.0, 10.0));
        assert_eq!(tracker.regions().len(), 2);

        // 3rd region exceeds capacity of 2; collapses to 1 bounding box
        tracker.add_region(RectF::new(50.0, 50.0, 10.0, 10.0));
        assert_eq!(tracker.regions().len(), 1);
        assert_eq!(tracker.bounding_box(), RectF::new(0.0, 0.0, 60.0, 60.0));
    }
}
