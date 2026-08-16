use crate::rendering::Color;
use serde::{Deserialize, Serialize};

/// Dynamic contrast guard computing relative luminance and WCAG 2.1 contrast ratios.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ContrastGuard;

impl ContrastGuard {
    /// Computes sRGB component relative luminance according to WCAG 2.1 spec:
    /// L = 0.2126 * R + 0.7152 * G + 0.0722 * B
    pub fn relative_luminance(color: &Color) -> f32 {
        let r = Self::linearize_component(color.r);
        let g = Self::linearize_component(color.g);
        let b = Self::linearize_component(color.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn linearize_component(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Computes WCAG 2.1 contrast ratio between two colors (range: 1.0 to 21.0).
    pub fn contrast_ratio(c1: &Color, c2: &Color) -> f32 {
        let l1 = Self::relative_luminance(c1);
        let l2 = Self::relative_luminance(c2);
        let (bright, dark) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (bright + 0.05) / (dark + 0.05)
    }

    /// Selects light or dark foreground color based on background ARGB (0xAARRGGBB).
    pub fn select_foreground_color(bg_argb: u32, light: Color, dark: Color) -> Color {
        let a = ((bg_argb >> 24) & 0xFF) as f32 / 255.0;
        let r = ((bg_argb >> 16) & 0xFF) as f32 / 255.0;
        let g = ((bg_argb >> 8) & 0xFF) as f32 / 255.0;
        let b = (bg_argb & 0xFF) as f32 / 255.0;
        let bg_color = Color::rgba(r, g, b, a);

        if Self::relative_luminance(&bg_color) < 0.5 {
            light
        } else {
            dark
        }
    }

    /// Ensures foreground color is crisp and legible over the given background color.
    /// If contrast ratio < 4.5:1 (WCAG AA), returns an automatically contrast-adjusted color.
    pub fn ensure_legible_fg(fg: &Color, bg: &Color) -> Color {
        let ratio = Self::contrast_ratio(fg, bg);
        if ratio >= 4.5 {
            *fg
        } else {
            // If background is dark, switch to crisp white/cyan; if light, switch to deep dark blue/black
            let bg_lum = Self::relative_luminance(bg);
            if bg_lum < 0.5 {
                Color::rgba(1.0, 1.0, 1.0, fg.a.max(0.9))
            } else {
                Color::rgba(0.05, 0.05, 0.08, fg.a.max(0.9))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_luminance_calculation() {
        let white = Color::rgba(1.0, 1.0, 1.0, 1.0);
        let black = Color::rgba(0.0, 0.0, 0.0, 1.0);

        assert!((ContrastGuard::relative_luminance(&white) - 1.0).abs() < 0.01);
        assert!((ContrastGuard::relative_luminance(&black) - 0.0).abs() < 0.01);

        let ratio = ContrastGuard::contrast_ratio(&white, &black);
        assert!((ratio - 21.0).abs() < 0.5);
    }

    #[test]
    fn test_contrast_guard_inverts_black_on_black() {
        let black_fg = Color::rgba(0.0, 0.0, 0.0, 1.0);
        let black_bg = Color::rgba(0.05, 0.05, 0.05, 1.0);

        let legible = ContrastGuard::ensure_legible_fg(&black_fg, &black_bg);
        assert!(legible.r > 0.8 && legible.g > 0.8 && legible.b > 0.8);
    }

    #[test]
    fn test_select_foreground_color() {
        let white = Color::rgba(1.0, 1.0, 1.0, 1.0);
        let black = Color::rgba(0.0, 0.0, 0.0, 1.0);

        let bg_dark = 0xFF111111;
        let selected = ContrastGuard::select_foreground_color(bg_dark, white, black);
        assert_eq!(selected, white);

        let bg_light = 0xFFEEEEEE;
        let selected_light = ContrastGuard::select_foreground_color(bg_light, white, black);
        assert_eq!(selected_light, black);
    }
}
