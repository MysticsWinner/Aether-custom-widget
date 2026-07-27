use serde::{Deserialize, Serialize};

/// Spring physics parameter configuration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl Default for SpringParams {
    fn default() -> Self {
        Self {
            stiffness: 180.0,
            damping: 12.0,
            mass: 1.0,
        }
    }
}

/// Physically-based Spring Animation solver.
#[derive(Debug, Clone)]
pub struct SpringAnimation {
    pub current: f32,
    pub target: f32,
    pub velocity: f32,
    pub params: SpringParams,
}

impl SpringAnimation {
    pub fn new(initial: f32, target: f32, params: SpringParams) -> Self {
        Self {
            current: initial,
            target,
            velocity: 0.0,
            params,
        }
    }

    /// Advances the spring simulation state by `delta_time_sec`.
    pub fn update(&mut self, dt: f32) -> f32 {
        let force = -self.params.stiffness * (self.current - self.target);
        let damping_force = -self.params.damping * self.velocity;
        let acceleration = (force + damping_force) / self.params.mass;

        self.velocity += acceleration * dt;
        self.current += self.velocity * dt;
        self.current
    }

    pub fn is_at_rest(&self) -> bool {
        (self.current - self.target).abs() < 0.001 && self.velocity.abs() < 0.001
    }
}

/// Standard Easing Curves.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EasingCurve {
    Linear,
    EaseInQuad,
    EaseOutQuad,
}

impl EasingCurve {
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingCurve::Linear => t,
            EasingCurve::EaseInQuad => t * t,
            EasingCurve::EaseOutQuad => t * (2.0 - t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_animation_convergence() {
        let mut spring = SpringAnimation::new(0.0, 100.0, SpringParams::default());
        let dt = 0.016; // 60 FPS tick

        for _ in 0..300 {
            spring.update(dt);
        }

        assert!(spring.is_at_rest() || (spring.current - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_easing_curve() {
        assert_eq!(EasingCurve::Linear.evaluate(0.5), 0.5);
        assert_eq!(EasingCurve::EaseInQuad.evaluate(0.5), 0.25);
        assert_eq!(EasingCurve::EaseOutQuad.evaluate(0.5), 0.75);
    }
}
