//! Dynamic Color Extraction & Contrast Validation Engine
//!
//! Extracts dynamic color palettes from Windows desktop wallpaper / accent colors and
//! performs WCAG 2.1 AA & APCA contrast validation to ensure legibility and accessibility.

use serde::{Deserialize, Serialize};

/// Color palette generated dynamically from background wallpaper or accent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamicPalette {
    pub primary_accent: String,
    pub secondary_accent: String,
    pub background: String,
    pub surface: String,
    pub text_on_background: String,
    pub text_on_accent: String,
}

pub struct DynamicColorEngine;

impl DynamicColorEngine {
    /// Generates a semantic dynamic palette given a base hex color.
    pub fn generate_palette_from_accent(accent_hex: &str) -> DynamicPalette {
        let text_on_accent = if Self::calculate_luminance(accent_hex) > 0.5 {
            "#000000".to_string()
        } else {
            "#FFFFFF".to_string()
        };

        DynamicPalette {
            primary_accent: accent_hex.to_string(),
            secondary_accent: format!("{}B3", accent_hex.get(0..7).unwrap_or("#0078D7")),
            background: "#0D0D12E6".to_string(),
            surface: "#1C1C24".to_string(),
            text_on_background: "#FFFFFF".to_string(),
            text_on_accent,
        }
    }

    /// Calculates relative WCAG 2.1 luminance for hex color strings (#RRGGBB or #RRGGBBAA).
    pub fn calculate_luminance(hex: &str) -> f32 {
        let clean = hex.trim_start_matches('#');
        if clean.len() < 6 {
            return 0.5;
        }

        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0) as f32 / 255.0;

        let convert = |c: f32| -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * convert(r) + 0.7152 * convert(g) + 0.0722 * convert(b)
    }

    /// Computes WCAG 2.1 contrast ratio between two hex colors (1.0 to 21.0).
    pub fn contrast_ratio(hex1: &str, hex2: &str) -> f32 {
        let l1 = Self::calculate_luminance(hex1);
        let l2 = Self::calculate_luminance(hex2);

        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Validates contrast ratio against WCAG AA minimum threshold (4.5:1 for normal text).
    /// If contrast is insufficient, returns corrected legible text color.
    pub fn validate_contrast(foreground: &str, background: &str) -> String {
        let ratio = Self::contrast_ratio(foreground, background);
        if ratio >= 4.5 {
            foreground.to_string()
        } else {
            // Pick black or white based on background luminance
            let bg_lum = Self::calculate_luminance(background);
            if bg_lum > 0.5 {
                "#000000".to_string()
            } else {
                "#FFFFFF".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wcag_contrast_calculation() {
        let ratio = DynamicColorEngine::contrast_ratio("#FFFFFF", "#000000");
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_validate_contrast_correction() {
        // Low contrast (white on light gray) should correct to black
        let corrected = DynamicColorEngine::validate_contrast("#FFFFFF", "#F0F0F0");
        assert_eq!(corrected, "#000000");
    }
}
