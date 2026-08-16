//! Semantic Typography Engine
//!
//! Provides typography role resolution (`caption`, `body`, `body_large`, `subtitle`,
//! `title`, `headline`, `display`, `numeric`, `monospace`) with DPI & accessibility scaling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypographyRole {
    Caption,
    Body,
    BodyLarge,
    Subtitle,
    Title,
    Headline,
    Display,
    Numeric,
    Monospace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypographySpec {
    pub family: String,
    pub fallback: String,
    pub size_pt: f32,
    pub weight: String,
    pub line_height: f32,
    pub letter_spacing: f32,
}

pub struct TypographyEngine;

impl TypographyEngine {
    /// Resolves semantic role to complete typography specification.
    pub fn resolve_role(role: TypographyRole, text_scale: f32) -> TypographySpec {
        let scale = text_scale.max(0.5).min(3.0);

        match role {
            TypographyRole::Caption => TypographySpec {
                family: "Segoe UI".to_string(),
                fallback: "Arial".to_string(),
                size_pt: 10.0 * scale,
                weight: "Normal".to_string(),
                line_height: 1.2,
                letter_spacing: 0.0,
            },
            TypographyRole::Body => TypographySpec {
                family: "Segoe UI".to_string(),
                fallback: "Arial".to_string(),
                size_pt: 14.0 * scale,
                weight: "Normal".to_string(),
                line_height: 1.4,
                letter_spacing: 0.0,
            },
            TypographyRole::BodyLarge => TypographySpec {
                family: "Segoe UI".to_string(),
                fallback: "Arial".to_string(),
                size_pt: 16.0 * scale,
                weight: "Normal".to_string(),
                line_height: 1.4,
                letter_spacing: 0.0,
            },
            TypographyRole::Subtitle => TypographySpec {
                family: "Segoe UI Variable Text".to_string(),
                fallback: "Segoe UI".to_string(),
                size_pt: 18.0 * scale,
                weight: "SemiBold".to_string(),
                line_height: 1.3,
                letter_spacing: 0.1,
            },
            TypographyRole::Title => TypographySpec {
                family: "Segoe UI Variable Display".to_string(),
                fallback: "Segoe UI".to_string(),
                size_pt: 24.0 * scale,
                weight: "Bold".to_string(),
                line_height: 1.2,
                letter_spacing: 0.2,
            },
            TypographyRole::Headline => TypographySpec {
                family: "Segoe UI Variable Display".to_string(),
                fallback: "Segoe UI".to_string(),
                size_pt: 32.0 * scale,
                weight: "Bold".to_string(),
                line_height: 1.1,
                letter_spacing: 0.2,
            },
            TypographyRole::Display => TypographySpec {
                family: "Segoe UI Variable Display".to_string(),
                fallback: "Segoe UI".to_string(),
                size_pt: 44.0 * scale,
                weight: "Bold".to_string(),
                line_height: 1.0,
                letter_spacing: 0.3,
            },
            TypographyRole::Numeric => TypographySpec {
                family: "Cascadia Code".to_string(),
                fallback: "Consolas".to_string(),
                size_pt: 20.0 * scale,
                weight: "Bold".to_string(),
                line_height: 1.0,
                letter_spacing: 0.5,
            },
            TypographyRole::Monospace => TypographySpec {
                family: "Consolas".to_string(),
                fallback: "Courier New".to_string(),
                size_pt: 12.0 * scale,
                weight: "Normal".to_string(),
                line_height: 1.3,
                letter_spacing: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typography_engine_role_scaling() {
        let base = TypographyEngine::resolve_role(TypographyRole::Body, 1.0);
        let scaled = TypographyEngine::resolve_role(TypographyRole::Body, 1.5);
        assert_eq!(base.size_pt, 14.0);
        assert_eq!(scaled.size_pt, 21.0);
    }
}
