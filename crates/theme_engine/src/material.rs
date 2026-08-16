//! Aether Material Engine & Surface Compositing Pipeline
//!
//! Provides material abstractions (`Solid`, `Transparent`, `Glass`, `Acrylic`, `Mica`, `Elevated`, `Custom`)
//! with dynamic hardware/battery/accessibility performance degradation fallbacks.

use serde::{Deserialize, Serialize};

/// Material types supported by Aether desktop compositing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    Solid,
    Transparent,
    Glass,
    Acrylic,
    Mica,
    Elevated,
    Custom,
}

impl Default for MaterialType {
    fn default() -> Self {
        MaterialType::Mica
    }
}

/// Visual surface parameters requested by widgets or containers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialSpec {
    pub material_type: MaterialType,
    pub tint_color: String,
    pub tint_opacity: f32,
    pub blur_radius: f32,
    pub luminosity: f32,
    pub noise_level: f32,
    pub border_highlight: bool,
    pub shadow_elevation: f32,
}

impl Default for MaterialSpec {
    fn default() -> Self {
        Self {
            material_type: MaterialType::Mica,
            tint_color: "#1E1E1E".to_string(),
            tint_opacity: 0.8,
            blur_radius: 30.0,
            luminosity: 0.9,
            noise_level: 0.02,
            border_highlight: true,
            shadow_elevation: 4.0,
        }
    }
}

/// Dynamic capability & system performance context for material resolution.
#[derive(Debug, Clone, Copy)]
pub struct MaterialContext {
    pub gpu_supports_blur: bool,
    pub is_battery_saver: bool,
    pub is_high_contrast: bool,
    pub is_reduce_transparency: bool,
}

impl Default for MaterialContext {
    fn default() -> Self {
        Self {
            gpu_supports_blur: true,
            is_battery_saver: false,
            is_high_contrast: false,
            is_reduce_transparency: false,
        }
    }
}

pub struct MaterialEngine;

impl MaterialEngine {
    /// Resolves requested `MaterialSpec` into effective rendering target based on system constraints.
    /// Gracefully degrades visual effects if GPU/Battery/Accessibility constraints require it.
    pub fn resolve_material(spec: &MaterialSpec, ctx: &MaterialContext) -> MaterialSpec {
        let mut resolved = spec.clone();

        // High contrast or transparency reduction override to solid/high contrast surface
        if ctx.is_high_contrast || ctx.is_reduce_transparency {
            resolved.material_type = MaterialType::Solid;
            resolved.tint_opacity = 1.0;
            resolved.blur_radius = 0.0;
            resolved.noise_level = 0.0;
            return resolved;
        }

        // Battery saver mode degrades blur and noise to save GPU power cycles
        if ctx.is_battery_saver {
            if resolved.material_type == MaterialType::Acrylic || resolved.material_type == MaterialType::Glass {
                resolved.material_type = MaterialType::Mica;
            }
            resolved.blur_radius = (resolved.blur_radius * 0.5).min(10.0);
            resolved.noise_level = 0.0;
        }

        // GPU unsupported blur degrades blur to zero solid transparent surface
        if !ctx.gpu_supports_blur && (resolved.material_type == MaterialType::Acrylic || resolved.material_type == MaterialType::Glass) {
            resolved.material_type = MaterialType::Transparent;
            resolved.blur_radius = 0.0;
        }

        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_engine_degradation_on_high_contrast() {
        let spec = MaterialSpec::default();
        let ctx = MaterialContext {
            is_high_contrast: true,
            ..Default::default()
        };

        let resolved = MaterialEngine::resolve_material(&spec, &ctx);
        assert_eq!(resolved.material_type, MaterialType::Solid);
        assert_eq!(resolved.tint_opacity, 1.0);
        assert_eq!(resolved.blur_radius, 0.0);
    }

    #[test]
    fn test_material_engine_battery_saver_degradation() {
        let spec = MaterialSpec {
            material_type: MaterialType::Acrylic,
            blur_radius: 40.0,
            ..Default::default()
        };
        let ctx = MaterialContext {
            is_battery_saver: true,
            ..Default::default()
        };

        let resolved = MaterialEngine::resolve_material(&spec, &ctx);
        assert_eq!(resolved.material_type, MaterialType::Mica);
        assert!(resolved.blur_radius <= 10.0);
    }
}
