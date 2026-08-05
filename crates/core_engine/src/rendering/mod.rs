pub mod benchmark;
pub mod d2d_renderer;
pub mod desktop_widget_window;
pub mod dirty_rect;
pub mod workerw;

pub use benchmark::{RainmeterBenchmark, RenderBenchmarkResult};
pub use d2d_renderer::Direct2DRenderer;
pub use desktop_widget_window::DesktopWidgetWindow;
pub use dirty_rect::DirtyRegionTracker;
pub use workerw::find_desktop_workerw_hwnd;

use std::fmt;

/// Floating point 2D rectangle representation used for geometry and invalidation bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RectF {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn intersects(&self, other: &RectF) -> bool {
        !(self.right() <= other.x
            || other.right() <= self.x
            || self.bottom() <= other.y
            || other.bottom() <= self.y)
    }

    pub fn union(&self, other: &RectF) -> RectF {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        RectF {
            x,
            y,
            width: right - x,
            height: bottom - y,
        }
    }
}

/// Floating point RGBA color token.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn transparent() -> Self {
        Self::rgba(0.0, 0.0, 0.0, 0.0)
    }

    pub fn black() -> Self {
        Self::rgba(0.0, 0.0, 0.0, 1.0)
    }
}

/// Target display refresh rate for V-Sync synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshRate {
    Hz60,
    Hz120,
    Hz144,
    Hz240,
    Custom(u32),
}

impl RefreshRate {
    pub fn target_fps(&self) -> u32 {
        match self {
            RefreshRate::Hz60 => 60,
            RefreshRate::Hz120 => 120,
            RefreshRate::Hz144 => 144,
            RefreshRate::Hz240 => 240,
            RefreshRate::Custom(hz) => *hz,
        }
    }

    pub fn frame_budget_us(&self) -> u64 {
        1_000_000 / (self.target_fps() as u64)
    }
}

impl fmt::Display for RefreshRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Hz", self.target_fps())
    }
}

/// Performance and render frame statistics.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    pub total_ticks: u64,
    pub rendered_frames: u64,
    pub skipped_frames: u64,
    pub dirty_rect_count: u64,
    pub last_frame_time_us: u64,
    pub total_frame_time_us: u64,
}

impl FrameStats {
    pub fn avg_frame_time_us(&self) -> f64 {
        if self.rendered_frames == 0 {
            0.0
        } else {
            self.total_frame_time_us as f64 / self.rendered_frames as f64
        }
    }
}

/// Abstract GPU Renderer Interface.
/// Enforces interface isolation so callers depend on abstractions rather than concrete hardware bindings.
pub trait GpuRenderer: Send + Sync {
    /// Initializes Direct3D / Direct2D / DirectComposition GPU device contexts.
    fn initialize(&mut self) -> anyhow::Result<()>;

    /// Invalidates a specific region on screen to force partial redraw.
    fn invalidate_region(&mut self, region: RectF);

    /// Begins frame pass; checks if dirty regions exist. Returns true if rendering is needed.
    fn begin_frame(&mut self) -> bool;

    /// Renders dirty regions onto Direct2D device context surfaces.
    fn draw_dirty_regions(&mut self) -> anyhow::Result<()>;

    /// Ends frame pass and presents swap chain buffers via DirectComposition.
    fn end_frame(&mut self) -> anyhow::Result<()>;

    /// Sets the target display refresh rate V-Sync interval.
    fn set_refresh_rate(&mut self, rate: RefreshRate);

    /// Returns telemetry and frame statistics.
    fn stats(&self) -> FrameStats;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectf_geometry() {
        let r1 = RectF::new(0.0, 0.0, 100.0, 100.0);
        let r2 = RectF::new(50.0, 50.0, 100.0, 100.0);
        let r3 = RectF::new(200.0, 200.0, 50.0, 50.0);

        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));

        let union_rect = r1.union(&r2);
        assert_eq!(union_rect, RectF::new(0.0, 0.0, 150.0, 150.0));
    }

    #[test]
    fn test_refresh_rate_budget() {
        assert_eq!(RefreshRate::Hz60.frame_budget_us(), 16666);
        assert_eq!(RefreshRate::Hz120.frame_budget_us(), 8333);
        assert_eq!(RefreshRate::Hz144.frame_budget_us(), 6944);
        assert_eq!(RefreshRate::Hz240.frame_budget_us(), 4166);
    }
}
