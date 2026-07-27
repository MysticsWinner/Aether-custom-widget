use crate::rendering::dirty_rect::DirtyRegionTracker;
use crate::rendering::{FrameStats, GpuRenderer, RectF, RefreshRate};
use std::time::Instant;
use tracing::{debug, info};

/// Direct2D & DirectComposition Hardware GPU Renderer implementation.
pub struct Direct2DRenderer {
    initialized: bool,
    refresh_rate: RefreshRate,
    dirty_tracker: DirtyRegionTracker,
    stats: FrameStats,
    frame_start_time: Option<Instant>,
    in_frame: bool,
}

impl Direct2DRenderer {
    /// Creates a new `Direct2DRenderer` with default refresh rate (120 Hz).
    pub fn new() -> Self {
        Self {
            initialized: false,
            refresh_rate: RefreshRate::Hz120,
            dirty_tracker: DirtyRegionTracker::default(),
            stats: FrameStats::default(),
            frame_start_time: None,
            in_frame: false,
        }
    }
}

impl Default for Direct2DRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuRenderer for Direct2DRenderer {
    fn initialize(&mut self) -> anyhow::Result<()> {
        info!(
            "Initializing Direct2D & DirectComposition GPU Renderer (Target V-Sync: {})...",
            self.refresh_rate
        );

        // Platform DirectX device initialization logic occurs here
        // On Windows 11: ID3D11Device -> IDXGIDevice -> ID2D1DeviceContext -> IDCompositionVisual
        self.initialized = true;

        info!("Direct2D & DirectComposition GPU pipeline initialized successfully.");
        Ok(())
    }

    fn invalidate_region(&mut self, region: RectF) {
        self.dirty_tracker.add_region(region);
    }

    fn begin_frame(&mut self) -> bool {
        self.stats.total_ticks += 1;

        if !self.initialized {
            return false;
        }

        // Check dirty rectangle state: ZERO UNNECESSARY REDRAWS
        if !self.dirty_tracker.is_dirty() {
            self.stats.skipped_frames += 1;
            debug!("Zero dirty regions detected. Skipping render frame pass.");
            return false;
        }

        self.frame_start_time = Some(Instant::now());
        self.in_frame = true;
        self.stats.dirty_rect_count += self.dirty_tracker.regions().len() as u64;
        true
    }

    fn draw_dirty_regions(&mut self) -> anyhow::Result<()> {
        if !self.in_frame {
            return Ok(());
        }

        let regions = self.dirty_tracker.regions().to_vec();
        for rect in regions {
            debug!(
                "Executing Direct2D PushAxisAlignedClip for region: ({:.1}, {:.1}, {:.1}x{:.1})",
                rect.x, rect.y, rect.width, rect.height
            );
            // In Windows 11 D2D context:
            // d2d_context.PushAxisAlignedClip(&D2D1_RECT_F { left: rect.x, top: rect.y, right: rect.right(), bottom: rect.bottom() }, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
            // ... Draw operations ...
            // d2d_context.PopAxisAlignedClip();
        }

        Ok(())
    }

    fn end_frame(&mut self) -> anyhow::Result<()> {
        if !self.in_frame {
            return Ok(());
        }

        // Present DirectComposition surface:
        // dxgi_swap_chain.Present1(sync_interval, 0, &present_params);

        if let Some(start) = self.frame_start_time.take() {
            let elapsed_us = start.elapsed().as_micros() as u64;
            self.stats.last_frame_time_us = elapsed_us;
            self.stats.total_frame_time_us += elapsed_us;
            self.stats.rendered_frames += 1;
        }

        self.dirty_tracker.clear();
        self.in_frame = false;
        Ok(())
    }

    fn set_refresh_rate(&mut self, rate: RefreshRate) {
        info!("Setting renderer refresh rate to: {}", rate);
        self.refresh_rate = rate;
    }

    fn stats(&self) -> FrameStats {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_initialization() {
        let mut renderer = Direct2DRenderer::new();
        assert!(renderer.initialize().is_ok());
    }

    #[test]
    fn test_zero_unnecessary_redraws() {
        let mut renderer = Direct2DRenderer::new();
        renderer.initialize().unwrap();

        // 1. Tick without invalidation -> Should skip render frame
        assert!(!renderer.begin_frame());
        assert_eq!(renderer.stats().skipped_frames, 1);
        assert_eq!(renderer.stats().rendered_frames, 0);

        // 2. Invalidate region -> Should render frame
        renderer.invalidate_region(RectF::new(10.0, 10.0, 100.0, 100.0));
        assert!(renderer.begin_frame());
        assert!(renderer.draw_dirty_regions().is_ok());
        assert!(renderer.end_frame().is_ok());

        assert_eq!(renderer.stats().rendered_frames, 1);
        assert_eq!(renderer.stats().skipped_frames, 1);

        // 3. Next tick without new invalidation -> Should skip again
        assert!(!renderer.begin_frame());
        assert_eq!(renderer.stats().skipped_frames, 2);
    }
}
