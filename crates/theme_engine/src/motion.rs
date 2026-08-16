//! Centralized Motion & Animation System
//!
//! Provides animation tokens, easing curves, spring physics configurations,
//! and accessibility motion intensity levels (`None`, `Reduced`, `Normal`, `Expressive`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionLevel {
    None,
    Reduced,
    Normal,
    Expressive,
}

impl Default for MotionLevel {
    fn default() -> Self {
        MotionLevel::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotionSpec {
    pub duration_ms: u32,
    pub easing: String,
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    pub intensity: f32,
}

pub struct MotionEngine;

impl MotionEngine {
    /// Resolves motion specification based on target motion level and user accessibility preferences.
    pub fn resolve_motion(base_duration_ms: u32, level: MotionLevel) -> MotionSpec {
        match level {
            MotionLevel::None => MotionSpec {
                duration_ms: 0,
                easing: "Linear".to_string(),
                stiffness: 1000.0,
                damping: 100.0,
                mass: 1.0,
                intensity: 0.0,
            },
            MotionLevel::Reduced => MotionSpec {
                duration_ms: (base_duration_ms as f32 * 0.5) as u32,
                easing: "EaseOutQuad".to_string(),
                stiffness: 300.0,
                damping: 30.0,
                mass: 0.8,
                intensity: 0.2,
            },
            MotionLevel::Normal => MotionSpec {
                duration_ms: base_duration_ms,
                easing: "EaseOutCubic".to_string(),
                stiffness: 180.0,
                damping: 18.0,
                mass: 1.0,
                intensity: 1.0,
            },
            MotionLevel::Expressive => MotionSpec {
                duration_ms: (base_duration_ms as f32 * 1.25) as u32,
                easing: "EaseOutBack".to_string(),
                stiffness: 140.0,
                damping: 12.0,
                mass: 1.1,
                intensity: 1.5,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motion_engine_reduced_motion() {
        let spec = MotionEngine::resolve_motion(300, MotionLevel::Reduced);
        assert_eq!(spec.duration_ms, 150);
        assert_eq!(spec.intensity, 0.2);
    }
}
