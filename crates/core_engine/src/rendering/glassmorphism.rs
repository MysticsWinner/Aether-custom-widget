//! Direct2D & Direct3D 11 Glassmorphism & Material Shader Pipeline
//!
//! Provides hardware-accelerated Mica, Acrylic, Gaussian blur, frosted glass,
//! and HDR dithering pipelines for glassmorphism desktop widgets.

use serde::{Deserialize, Serialize};
use theme_engine::MaterialSpec;
use tracing::debug;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas, RenderEffect};

/// Material effect type for shader composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlassEffectType {
    /// Windows 11 Mica material (wallpaper tinted, subtle reflection)
    Mica,
    /// Windows 11 Acrylic material (desktop background blur with noise texture)
    Acrylic,
    /// Frosted glass (high-radius dual-pass Gaussian blur)
    FrostedGlass,
    /// Neon glow accent effect
    NeonGlow,
}

/// Glassmorphism rendering configuration and dithering pipeline.
#[derive(Debug, Clone)]
pub struct GlassmorphismPipeline {
    enable_dithering: bool,
    fallback_mode: bool,
}

impl GlassmorphismPipeline {
    /// Creates a new `GlassmorphismPipeline`.
    pub fn new() -> Self {
        Self {
            enable_dithering: true,
            fallback_mode: false,
        }
    }

    /// Sets whether noise dithering is enabled (prevents 8-bit color banding on dark glass).
    pub fn with_dithering(mut self, enable: bool) -> Self {
        self.enable_dithering = enable;
        self
    }

    /// Sets whether fallback rendering mode is active (for compatibility or low-end hardware).
    pub fn with_fallback_mode(mut self, fallback: bool) -> Self {
        self.fallback_mode = fallback;
        self
    }

    /// Returns whether fallback rendering mode is active.
    pub fn is_fallback_mode(&self) -> bool {
        self.fallback_mode
    }

    /// Applies the specified `MaterialSpec` onto the render canvas.
    pub fn apply_material(
        &self,
        canvas: &mut BatchRenderCanvas,
        rect: RectF,
        spec: &MaterialSpec,
    ) {
        debug!("Applying material {:?} to rect {:?}", spec.material_type, rect);

        let blur_radius = spec.blur_radius.max(5.0);

        // 1. Emit Blur Effect Pass
        canvas.draw_effect(
            RenderEffect::GaussianBlur {
                radius: blur_radius,
            },
            rect,
        );

        // 2. Parse Tint Color
        let tint = parse_hex_color(&spec.tint_color, spec.tint_opacity);

        // 3. Emit Background Tint Rectangle
        canvas.draw_rect(rect, tint, 16.0);

        // 4. Emit Border / Outline
        if spec.border_highlight {
            let border_color = Color::rgba(1.0, 1.0, 1.0, 0.15);
            canvas.draw_rect(rect, border_color, 16.0);
            debug!("Applied border highlight {:?}", border_color);
        }
    }

    /// Computes perceived luminance of a Color (Rec. 709 formula: Y = 0.2126 R + 0.7152 G + 0.0722 B).
    pub fn compute_luminance(color: Color) -> f32 {
        0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
    }

    /// Generates a fast 4x4 Bayer ordered dither matrix value for spatial anti-banding.
    pub fn bayer_dither_4x4(x: u32, y: u32) -> f32 {
        const MATRIX: [f32; 16] = [
            0.0 / 16.0,  8.0 / 16.0,  2.0 / 16.0, 10.0 / 16.0,
            12.0 / 16.0,  4.0 / 16.0, 14.0 / 16.0,  6.0 / 16.0,
             3.0 / 16.0, 11.0 / 16.0,  1.0 / 16.0,  9.0 / 16.0,
            15.0 / 16.0,  7.0 / 16.0, 13.0 / 16.0,  5.0 / 16.0,
        ];
        let idx = ((y % 4) * 4 + (x % 4)) as usize;
        MATRIX[idx] - 0.5
    }
}

impl Default for GlassmorphismPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to parse hex color strings (e.g. `#1E1E1E` or `#FFFFFF`) into `Color`.
fn parse_hex_color(hex: &str, opacity: f32) -> Color {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return Color::rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, opacity);
        }
    }
    Color::rgba(0.1, 0.1, 0.15, opacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use theme_engine::MaterialType;

    #[test]
    fn test_glassmorphism_pipeline_initialization() {
        let pipeline = GlassmorphismPipeline::new().with_dithering(true);
        assert!(pipeline.enable_dithering);
        assert!(!pipeline.fallback_mode);
    }

    #[test]
    fn test_compute_luminance() {
        let white = Color::rgb(1.0, 1.0, 1.0);
        let black = Color::rgb(0.0, 0.0, 0.0);
        let green = Color::rgb(0.0, 1.0, 0.0);

        assert!((GlassmorphismPipeline::compute_luminance(white) - 1.0).abs() < 0.001);
        assert_eq!(GlassmorphismPipeline::compute_luminance(black), 0.0);
        assert!((GlassmorphismPipeline::compute_luminance(green) - 0.7152).abs() < 0.001);
    }

    #[test]
    fn test_bayer_dither_matrix() {
        let d0 = GlassmorphismPipeline::bayer_dither_4x4(0, 0);
        let d1 = GlassmorphismPipeline::bayer_dither_4x4(1, 0);
        assert!(d0 >= -0.5 && d0 <= 0.5);
        assert!(d1 >= -0.5 && d1 <= 0.5);
        assert_ne!(d0, d1);
    }

    #[test]
    fn test_apply_material_generates_draw_commands() {
        let pipeline = GlassmorphismPipeline::new();
        let mut canvas = BatchRenderCanvas::new();
        let spec = MaterialSpec {
            material_type: MaterialType::Mica,
            tint_color: "#1E1E24".to_string(),
            tint_opacity: 0.85,
            blur_radius: 25.0,
            border_highlight: true,
            ..Default::default()
        };

        pipeline.apply_material(&mut canvas, RectF::new(0.0, 0.0, 300.0, 150.0), &spec);
        assert!(canvas.commands().len() >= 2);
    }
}
