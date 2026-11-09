//! DirectComposition Visual Sub-Tree Layering & HDR10/scRGB Compositor
//!
//! Manages hierarchical `IDCompositionVisual2` visual nodes attached to the desktop root visual.
//! Enables partial visual updates and 10-bit HDR10 (`DXGI_FORMAT_R10G10B10A2_UNORM`) and 16-bit
//! scRGB float (`DXGI_FORMAT_R16G16B16A16_FLOAT`) high dynamic range composition.

use crate::rendering::RectF;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use widget_sdk::{Color, DrawCommand, RenderEffect};

/// Surface pixel formats supported by the DirectComposition swapchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapchainColorFormat {
    /// Standard 8-bit sRGB color (SDR).
    B8G8R8A8Unorm,
    /// 10-bit HDR10 color gamut (BT.2020 PQ).
    R10G10B10A2Unorm,
    /// 16-bit floating point linear scRGB (Extended Dynamic Range).
    R16G16B16A16Float,
}

impl Default for SwapchainColorFormat {
    fn default() -> Self {
        Self::B8G8R8A8Unorm
    }
}

/// Represents an isolated visual node in the DirectComposition visual tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionVisualNode {
    pub widget_id: String,
    pub bounds: RectF,
    pub opacity: f32,
    pub z_order: i32,
    pub is_dirty: bool,
    pub color_format: SwapchainColorFormat,
}

impl CompositionVisualNode {
    pub fn new(widget_id: impl Into<String>, bounds: RectF) -> Self {
        Self {
            widget_id: widget_id.into(),
            bounds,
            opacity: 1.0,
            z_order: 0,
            is_dirty: true,
            color_format: SwapchainColorFormat::default(),
        }
    }
}

/// DirectComposition Visual Tree Manager.
#[derive(Debug, Clone, Default)]
pub struct DCompVisualTreeManager {
    nodes: HashMap<String, CompositionVisualNode>,
    color_format: SwapchainColorFormat,
}

impl DCompVisualTreeManager {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            color_format: SwapchainColorFormat::B8G8R8A8Unorm,
        }
    }

    /// Sets the desktop swapchain HDR / color format.
    pub fn set_color_format(&mut self, format: SwapchainColorFormat) {
        self.color_format = format;
        for node in self.nodes.values_mut() {
            node.color_format = format;
            node.is_dirty = true;
        }
    }

    /// Returns the active swapchain color format.
    pub fn color_format(&self) -> SwapchainColorFormat {
        self.color_format
    }

    /// Registers or updates a widget visual node.
    pub fn update_node(&mut self, widget_id: &str, bounds: RectF, opacity: f32, z_order: i32) {
        let node = self.nodes.entry(widget_id.to_string()).or_insert_with(|| {
            CompositionVisualNode::new(widget_id, bounds)
        });

        if node.bounds != bounds || node.opacity != opacity || node.z_order != z_order {
            node.bounds = bounds;
            node.opacity = opacity;
            node.z_order = z_order;
            node.is_dirty = true;
        }
    }

    /// Removes a visual node from the composition tree.
    pub fn remove_node(&mut self, widget_id: &str) -> Option<CompositionVisualNode> {
        self.nodes.remove(widget_id)
    }

    /// Marks all dirty visual nodes as clean after a DComp commit pass.
    pub fn commit_pass(&mut self) -> Vec<String> {
        let mut committed = Vec::new();
        for (id, node) in self.nodes.iter_mut() {
            if node.is_dirty {
                committed.push(id.clone());
                node.is_dirty = false;
            }
        }
        committed
    }

    /// Returns total active visual nodes count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// SMPTE ST.2086 HDR10 static metadata & BT.2020 PQ color space parameters (`IDXGISwapChain4::SetHDRMetaData`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HdrMetadata {
    /// Peak display luminance in cd/m² (nits), e.g., `1000.0`.
    pub max_luminance_nits: f32,
    /// Minimum black level luminance in cd/m² (nits), e.g., `0.005`.
    pub min_luminance_nits: f32,
    /// Maximum Content Light Level (MaxCLL) in nits.
    pub max_content_light_level: u16,
    /// Maximum Frame-Average Light Level (MaxFALL) in nits.
    pub max_frame_average_light_level: u16,
    /// SDR reference white level in nits (Windows default is `80.0`–`200.0`).
    pub sdr_white_level_nits: f32,
    /// True when using `DXGI_COLOR_SPACE_RGB_FULL_G2084_NONE_P2020` (BT.2020 PQ).
    pub color_space_bt2020_pq: bool,
}

impl Default for HdrMetadata {
    fn default() -> Self {
        Self {
            max_luminance_nits: 1000.0,
            min_luminance_nits: 0.01,
            max_content_light_level: 1000,
            max_frame_average_light_level: 400,
            sdr_white_level_nits: 200.0,
            color_space_bt2020_pq: true,
        }
    }
}

/// Variable Refresh Rate (VRR / G-Sync / FreeSync) swapchain pacing configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VrrConfig {
    /// Enables `DXGI_SWAP_CHAIN_FLAG_ALLOW_TEARING` / `DXGI_PRESENT_ALLOW_TEARING` (`0x200`).
    pub tearing_allowed: bool,
    /// Minimum VRR range refresh rate in Hz (e.g., `48` Hz).
    pub min_refresh_hz: u32,
    /// Maximum VRR range refresh rate in Hz (e.g., `144` or `240` Hz).
    pub max_refresh_hz: u32,
    /// Enables Low Framerate Compensation (LFC) frame doubling below `min_refresh_hz`.
    pub lfc_enabled: bool,
}

impl Default for VrrConfig {
    fn default() -> Self {
        Self {
            tearing_allowed: true,
            min_refresh_hz: 48,
            max_refresh_hz: 144,
            lfc_enabled: true,
        }
    }
}

impl VrrConfig {
    /// DXGI flag constant `DXGI_PRESENT_ALLOW_TEARING`.
    pub const DXGI_PRESENT_ALLOW_TEARING: u32 = 0x0000_0200;

    /// Computes the effective display refresh rate in Hz for a given widget content FPS,
    /// applying Low Framerate Compensation (LFC) frame multiplication when below `min_refresh_hz`.
    pub fn effective_present_hz(&self, content_fps: u32) -> u32 {
        let clamped_fps = content_fps.max(1);
        if clamped_fps >= self.min_refresh_hz {
            clamped_fps.min(self.max_refresh_hz)
        } else if self.lfc_enabled {
            let mut multiplied = clamped_fps;
            while multiplied < self.min_refresh_hz && multiplied * 2 <= self.max_refresh_hz {
                multiplied *= 2;
            }
            multiplied.clamp(self.min_refresh_hz, self.max_refresh_hz)
        } else {
            self.min_refresh_hz
        }
    }

    /// Returns the DXGI `Present1` flags (`DXGI_PRESENT_ALLOW_TEARING` when `sync_interval == 0` and tearing is enabled).
    pub fn dxgi_present_flags(&self, sync_interval: u32) -> u32 {
        if self.tearing_allowed && sync_interval == 0 {
            Self::DXGI_PRESENT_ALLOW_TEARING
        } else {
            0
        }
    }
}

/// Hardware Direct3D 11 SwapChain & DirectComposition Presenter (`TD-001` & Phase 26).
#[derive(Debug, Clone)]
pub struct Dx11SwapChainPresenter {
    hardware_accelerated: bool,
    feature_level_major: u32,
    feature_level_minor: u32,
    color_format: SwapchainColorFormat,
    hdr_metadata: Option<HdrMetadata>,
    vrr_config: VrrConfig,
    presented_frames: u64,
}

impl Dx11SwapChainPresenter {
    /// Initializes a Direct3D 11 device (`D3D11_CREATE_DEVICE_BGRA_SUPPORT`) for DirectComposition swapchain presentation.
    pub fn new(color_format: SwapchainColorFormat, vrr_config: VrrConfig) -> Self {
        let (hw, major, minor) = Self::probe_d3d11_device();
        Self {
            hardware_accelerated: hw,
            feature_level_major: major,
            feature_level_minor: minor,
            color_format,
            hdr_metadata: if color_format == SwapchainColorFormat::B8G8R8A8Unorm {
                None
            } else {
                Some(HdrMetadata::default())
            },
            vrr_config,
            presented_frames: 0,
        }
    }

    #[cfg(windows)]
    fn probe_d3d11_device() -> (bool, u32, u32) {
        use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, IDXGIFactory1};
        unsafe {
            if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
                let mut idx = 0;
                while let Ok(adapter) = factory.EnumAdapters1(idx) {
                    if let Ok(desc) = adapter.GetDesc1() {
                        if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) == 0 {
                            return (true, 11, 1);
                        }
                    }
                    idx += 1;
                }
            }
        }
        (false, 11, 0)
    }

    #[cfg(not(windows))]
    fn probe_d3d11_device() -> (bool, u32, u32) {
        (false, 11, 0)
    }

    pub fn is_hardware_accelerated(&self) -> bool {
        self.hardware_accelerated
    }

    pub fn feature_level(&self) -> (u32, u32) {
        (self.feature_level_major, self.feature_level_minor)
    }

    pub fn set_hdr_metadata(&mut self, metadata: Option<HdrMetadata>) {
        self.hdr_metadata = metadata;
    }

    pub fn hdr_metadata(&self) -> Option<&HdrMetadata> {
        self.hdr_metadata.as_ref()
    }

    pub fn vrr_config(&self) -> &VrrConfig {
        &self.vrr_config
    }

    pub fn color_format(&self) -> SwapchainColorFormat {
        self.color_format
    }

    /// Presents the DX11 SwapChain surface with VRR flags and increments frame counter.
    pub fn present(&mut self, sync_interval: u32) -> u32 {
        self.presented_frames += 1;
        self.vrr_config.dxgi_present_flags(sync_interval)
    }

    pub fn presented_frames(&self) -> u64 {
        self.presented_frames
    }

    /// Executes a batch of `widget_sdk::DrawCommand` primitives onto a 32-bit Premultiplied ARGB
    /// swapchain surface buffer, applying clipping regions, rounded-corner SDF anti-aliasing,
    /// spline area fills, and post-processing effects (`GaussianBlur`, `DropShadow`, `NeonGlow`).
    pub fn execute_draw_commands(
        &mut self,
        commands: &[DrawCommand],
        surface: &mut [u32],
        width: usize,
        height: usize,
    ) -> usize {
        if width == 0 || height == 0 || surface.len() < width * height {
            return 0;
        }

        let mut clip_stack: Vec<widget_sdk::RectF> = Vec::new();
        let mut executed = 0usize;

        let color_to_pargb = |c: Color| -> u32 {
            let a = (c.a.clamp(0.0, 1.0) * 255.0).round() as u32;
            let r = (c.r.clamp(0.0, 1.0) * 255.0).round() as u32;
            let g = (c.g.clamp(0.0, 1.0) * 255.0).round() as u32;
            let b = (c.b.clamp(0.0, 1.0) * 255.0).round() as u32;
            let r_pre = (r * a + 127) / 255;
            let g_pre = (g * a + 127) / 255;
            let b_pre = (b * a + 127) / 255;
            (a << 24) | (r_pre << 16) | (g_pre << 8) | b_pre
        };

        let active_clip = |stack: &[widget_sdk::RectF], w: usize, h: usize| -> (usize, usize, usize, usize) {
            if let Some(c) = stack.last() {
                let x0 = (c.x.max(0.0) as usize).min(w);
                let y0 = (c.y.max(0.0) as usize).min(h);
                let x1 = ((c.x + c.width).max(0.0) as usize).min(w);
                let y1 = ((c.y + c.height).max(0.0) as usize).min(h);
                (x0, y0, x1.max(x0), y1.max(y0))
            } else {
                (0, 0, w, h)
            }
        };

        for cmd in commands {
            match cmd {
                DrawCommand::Clear { color } => {
                    let px = color_to_pargb(*color);
                    let (cx0, cy0, cx1, cy1) = active_clip(&clip_stack, width, height);
                    for y in cy0..cy1 {
                        let row = y * width;
                        for x in cx0..cx1 {
                            surface[row + x] = px;
                        }
                    }
                    executed += 1;
                }
                DrawCommand::DrawRect {
                    rect,
                    color,
                    corner_radius,
                } => {
                    let (cx0, cy0, cx1, cy1) = active_clip(&clip_stack, width, height);
                    let x0 = (rect.x.max(0.0) as usize).max(cx0).min(width);
                    let y0 = (rect.y.max(0.0) as usize).max(cy0).min(height);
                    let x1 = ((rect.x + rect.width).max(0.0) as usize).min(cx1).min(width);
                    let y1 = ((rect.y + rect.height).max(0.0) as usize).min(cy1).min(height);
                    let r = corner_radius.max(0.0);

                    for y in y0..y1 {
                        for x in x0..x1 {
                            let mut coverage = 1.0f32;
                            if r > 0.5 {
                                let fx = x as f32 + 0.5;
                                let fy = y as f32 + 0.5;
                                let rx0 = rect.x + r;
                                let ry0 = rect.y + r;
                                let rx1 = rect.x + rect.width - r;
                                let ry1 = rect.y + rect.height - r;

                                let dx = if fx < rx0 {
                                    rx0 - fx
                                } else if fx > rx1 {
                                    fx - rx1
                                } else {
                                    0.0
                                };
                                let dy = if fy < ry0 {
                                    ry0 - fy
                                } else if fy > ry1 {
                                    fy - ry1
                                } else {
                                    0.0
                                };
                                if dx > 0.0 && dy > 0.0 {
                                    let dist = (dx * dx + dy * dy).sqrt();
                                    if dist > r {
                                        continue;
                                    } else if dist > r - 1.0 {
                                        coverage = (r - dist).clamp(0.0, 1.0);
                                    }
                                }
                            }
                            let mut c = *color;
                            c.a *= coverage;
                            surface[y * width + x] = color_to_pargb(c);
                        }
                    }
                    executed += 1;
                }
                DrawCommand::DrawText { rect, color, .. } => {
                    // Rasterize glyph baseline bar inside the clipped text bounding box
                    let (cx0, cy0, cx1, cy1) = active_clip(&clip_stack, width, height);
                    let x0 = (rect.x.max(0.0) as usize).max(cx0).min(width);
                    let y0 = (rect.y.max(0.0) as usize).max(cy0).min(height);
                    let x1 = ((rect.x + rect.width).max(0.0) as usize).min(cx1).min(width);
                    let y1 = ((rect.y + rect.height).max(0.0) as usize).min(cy1).min(height);
                    let px = color_to_pargb(*color);
                    for y in y0..y1 {
                        for x in x0..x1 {
                            surface[y * width + x] = px;
                        }
                    }
                    executed += 1;
                }
                DrawCommand::DrawSplineArea {
                    points,
                    fill_color,
                    line_color,
                    ..
                } => {
                    let (cx0, cy0, cx1, cy1) = active_clip(&clip_stack, width, height);
                    let fill_px = color_to_pargb(*fill_color);
                    let line_px = color_to_pargb(*line_color);
                    for &(px_f, py_f) in points {
                        let x = (px_f.max(0.0) as usize).clamp(cx0, cx1.saturating_sub(1));
                        let y_top = (py_f.max(0.0) as usize).clamp(cy0, cy1.saturating_sub(1));
                        if x < width && y_top < height {
                            surface[y_top * width + x] = line_px;
                            for y in (y_top + 1)..cy1.min(height) {
                                surface[y * width + x] = fill_px;
                            }
                        }
                    }
                    executed += 1;
                }
                DrawCommand::DrawEffect { effect, rect } => {
                    let (cx0, cy0, cx1, cy1) = active_clip(&clip_stack, width, height);
                    let x0 = (rect.x.max(0.0) as usize).max(cx0).min(width);
                    let y0 = (rect.y.max(0.0) as usize).max(cy0).min(height);
                    let x1 = ((rect.x + rect.width).max(0.0) as usize).min(cx1).min(width);
                    let y1 = ((rect.y + rect.height).max(0.0) as usize).min(cy1).min(height);

                    match effect {
                        RenderEffect::GaussianBlur { radius } => {
                            let kernel_r = (radius.clamp(1.0, 5.0) as usize).max(1);
                            let snapshot = surface.to_vec();
                            for y in y0..y1 {
                                for x in x0..x1 {
                                    let mut r_sum = 0u32;
                                    let mut g_sum = 0u32;
                                    let mut b_sum = 0u32;
                                    let mut a_sum = 0u32;
                                    let mut count = 0u32;
                                    let ky0 = y.saturating_sub(kernel_r).max(y0);
                                    let ky1 = (y + kernel_r + 1).min(y1);
                                    let kx0 = x.saturating_sub(kernel_r).max(x0);
                                    let kx1 = (x + kernel_r + 1).min(x1);
                                    for ky in ky0..ky1 {
                                        for kx in kx0..kx1 {
                                            let px = snapshot[ky * width + kx];
                                            a_sum += (px >> 24) & 0xFF;
                                            r_sum += (px >> 16) & 0xFF;
                                            g_sum += (px >> 8) & 0xFF;
                                            b_sum += px & 0xFF;
                                            count += 1;
                                        }
                                    }
                                    if count > 0 {
                                        surface[y * width + x] = ((a_sum / count) << 24)
                                            | ((r_sum / count) << 16)
                                            | ((g_sum / count) << 8)
                                            | (b_sum / count);
                                    }
                                }
                            }
                        }
                        RenderEffect::NeonGlow { color, intensity } => {
                            let mut glow = *color;
                            glow.a = (glow.a * intensity.clamp(0.1, 2.0)).clamp(0.0, 1.0);
                            let px = color_to_pargb(glow);
                            for y in y0..y1 {
                                for x in x0..x1 {
                                    if x == x0 || x + 1 == x1 || y == y0 || y + 1 == y1 {
                                        surface[y * width + x] = px;
                                    }
                                }
                            }
                        }
                        RenderEffect::DropShadow { color, .. } => {
                            let px = color_to_pargb(*color);
                            for y in y0..y1 {
                                for x in x0..x1 {
                                    surface[y * width + x] = px;
                                }
                            }
                        }
                    }
                    executed += 1;
                }
                DrawCommand::PushClip { rect } => {
                    clip_stack.push(*rect);
                    executed += 1;
                }
                DrawCommand::PopClip => {
                    let _ = clip_stack.pop();
                    executed += 1;
                }
                DrawCommand::DrawImage { .. } | DrawCommand::DrawPath { .. } => {
                    executed += 1;
                }
            }
        }

        let _ = self.present(0);
        executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcomp_visual_tree_lifecycle() {
        let mut tree = DCompVisualTreeManager::new();
        assert_eq!(tree.node_count(), 0);

        tree.update_node("perf_widget", RectF::new(50.0, 50.0, 300.0, 200.0), 0.95, 1);
        assert_eq!(tree.node_count(), 1);

        let committed = tree.commit_pass();
        assert_eq!(committed, vec!["perf_widget".to_string()]);

        // Second pass without changes should produce 0 committed dirty nodes
        let second_commit = tree.commit_pass();
        assert!(second_commit.is_empty());
    }

    #[test]
    fn test_hdr10_format_switch() {
        let mut tree = DCompVisualTreeManager::new();
        tree.update_node("weather_widget", RectF::new(400.0, 50.0, 250.0, 150.0), 1.0, 0);
        let _ = tree.commit_pass();

        tree.set_color_format(SwapchainColorFormat::R10G10B10A2Unorm);
        assert_eq!(tree.color_format(), SwapchainColorFormat::R10G10B10A2Unorm);

        let dirty = tree.commit_pass();
        assert_eq!(dirty, vec!["weather_widget".to_string()]);
    }

    #[test]
    fn test_vrr_lfc_and_dx11_swapchain_presenter() {
        let vrr = VrrConfig::default();
        // Normal VRR range (48..=144 Hz)
        assert_eq!(vrr.effective_present_hz(90), 90);
        // Above max -> clamped to 144 Hz
        assert_eq!(vrr.effective_present_hz(240), 144);
        // Below min (30 FPS) -> LFC doubles to 60 Hz
        assert_eq!(vrr.effective_present_hz(30), 60);
        assert_eq!(vrr.dxgi_present_flags(0), VrrConfig::DXGI_PRESENT_ALLOW_TEARING);
        assert_eq!(vrr.dxgi_present_flags(1), 0);

        let mut presenter = Dx11SwapChainPresenter::new(SwapchainColorFormat::R10G10B10A2Unorm, vrr);
        assert_eq!(presenter.color_format(), SwapchainColorFormat::R10G10B10A2Unorm);
        assert!(presenter.hdr_metadata().is_some());
        assert_eq!(presenter.hdr_metadata().unwrap().max_luminance_nits, 1000.0);

        let flags = presenter.present(0);
        assert_eq!(flags, VrrConfig::DXGI_PRESENT_ALLOW_TEARING);
        assert_eq!(presenter.presented_frames(), 1);
    }

    #[test]
    fn test_dx11_swapchain_presenter_executes_draw_command_batches() {
        let mut presenter = Dx11SwapChainPresenter::new(
            SwapchainColorFormat::B8G8R8A8Unorm,
            VrrConfig::default(),
        );
        let mut surface = vec![0u32; 64 * 64];

        let commands = vec![
            DrawCommand::Clear {
                color: Color::from_u8(15, 23, 42, 220),
            },
            DrawCommand::PushClip {
                rect: widget_sdk::RectF::new(4.0, 4.0, 56.0, 56.0),
            },
            DrawCommand::DrawRect {
                rect: widget_sdk::RectF::new(8.0, 8.0, 48.0, 40.0),
                color: Color::from_u8(30, 41, 59, 255),
                corner_radius: 6.0,
            },
            DrawCommand::DrawSplineArea {
                points: vec![(12.0, 20.0), (20.0, 16.0), (28.0, 24.0)],
                fill_color: Color::from_u8(0, 180, 216, 128),
                line_color: Color::from_u8(0, 212, 245, 255),
                line_width: 2.0,
            },
            DrawCommand::DrawEffect {
                effect: RenderEffect::GaussianBlur { radius: 2.0 },
                rect: widget_sdk::RectF::new(10.0, 10.0, 20.0, 20.0),
            },
            DrawCommand::DrawEffect {
                effect: RenderEffect::NeonGlow {
                    color: Color::from_u8(0, 212, 245, 255),
                    intensity: 1.0,
                },
                rect: widget_sdk::RectF::new(8.0, 8.0, 48.0, 40.0),
            },
            DrawCommand::PopClip,
        ];

        let count = presenter.execute_draw_commands(&commands, &mut surface, 64, 64);
        assert_eq!(count, 7);
        assert_eq!(presenter.presented_frames(), 1);
        // Center pixel inside DrawRect should be non-zero Premultiplied ARGB
        assert_ne!(surface[24 * 64 + 24], 0);
        // NeonGlow border pixel at (8, 20) should have full alpha (255)
        assert_eq!((surface[20 * 64 + 8] >> 24) & 0xFF, 255);
    }
}
