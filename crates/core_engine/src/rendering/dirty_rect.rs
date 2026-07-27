use crate::rendering::RectF;
use tracing::debug;

/// Tracks invalidated screen bounds to enforce dirty rectangle partial rendering.
/// Ensures zero unnecessary redraws when screen contents are static.
#[derive(Debug, Clone)]
pub struct DirtyRegionTracker {
    dirty_regions: Vec<RectF>,
    max_regions: usize,
}

impl DirtyRegionTracker {
    /// Creates a new `DirtyRegionTracker` with a maximum tracking capacity.
    pub fn new(max_regions: usize) -> Self {
        Self {
            dirty_regions: Vec::with_capacity(max_regions),
            max_regions,
        }
    }

    /// Adds a region to be invalidated for partial redraw.
    pub fn add_region(&mut self, rect: RectF) {
        if rect.is_empty() {
            return;
        }

        debug!("Invalidating dirty region: ({:.1}, {:.1}, {:.1}x{:.1})", rect.x, rect.y, rect.width, rect.height);

        // Merge with existing intersecting regions if possible to reduce draw calls
        for region in self.dirty_regions.iter_mut() {
            if region.intersects(&rect) {
                *region = region.union(&rect);
                return;
            }
        }

        if self.dirty_regions.len() >= self.max_regions {
            // If capacity limit reached, merge into bounding box
            let combined = self.bounding_box();
            self.dirty_regions.clear();
            self.dirty_regions.push(combined.union(&rect));
        } else {
            self.dirty_regions.push(rect);
        }
    }

    /// Returns whether any dirty regions are queued for rendering.
    pub fn is_dirty(&self) -> bool {
        !self.dirty_regions.is_empty()
    }

    /// Returns a slice of all active dirty regions.
    pub fn regions(&self) -> &[RectF] {
        &self.dirty_regions
    }

    /// Computes the single bounding rectangle containing all dirty regions.
    pub fn bounding_box(&self) -> RectF {
        if self.dirty_regions.is_empty() {
            return RectF::zero();
        }
        let mut bbox = self.dirty_regions[0];
        for rect in &self.dirty_regions[1..] {
            bbox = bbox.union(rect);
        }
        bbox
    }

    /// Clears all dirty regions after a successful render frame pass.
    pub fn clear(&mut self) {
        self.dirty_regions.clear();
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
        let mut tracker = DirtyRegionTracker::default();
        assert!(!tracker.is_dirty());
        assert_eq!(tracker.regions().len(), 0);
    }

    #[test]
    fn test_dirty_tracker_merge_overlapping() {
        let mut tracker = DirtyRegionTracker::default();
        tracker.add_region(RectF::new(10.0, 10.0, 50.0, 50.0));
        tracker.add_region(RectF::new(30.0, 30.0, 50.0, 50.0));

        assert!(tracker.is_dirty());
        assert_eq!(tracker.regions().len(), 1);
        assert_eq!(tracker.bounding_box(), RectF::new(10.0, 10.0, 70.0, 70.0));
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
}
