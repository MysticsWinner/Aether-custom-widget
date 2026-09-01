use std::f32::consts::PI;

/// Easing functions for dynamic UI transitions and micro-animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingCurve {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseOutElastic,
    EaseOutBounce,
}

impl EasingCurve {
    /// Evaluates easing curve progress `t` in range `[0.0, 1.0]`.
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingCurve::Linear => t,
            EasingCurve::EaseInQuad => t * t,
            EasingCurve::EaseOutQuad => t * (2.0 - t),
            EasingCurve::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingCurve::EaseInCubic => t * t * t,
            EasingCurve::EaseOutCubic => {
                let p = t - 1.0;
                p * p * p + 1.0
            }
            EasingCurve::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let f = 2.0 * t - 2.0;
                    0.5 * f * f * f + 1.0
                }
            }
            EasingCurve::EaseOutElastic => {
                let p = 0.3;
                2.0f32.powf(-10.0 * t) * ((t - p / 4.0) * (2.0 * PI) / p).sin() + 1.0
            }
            EasingCurve::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
        }
    }
}

/// A keyframe specifying a target value at a normalized point in time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub time_fraction: f32,
    pub value: f32,
}

/// Keyframe-based animation track with easing curve interpolation.
#[derive(Debug, Clone)]
pub struct KeyframeAnimation {
    pub start_value: f32,
    pub end_value: f32,
    pub duration_secs: f32,
    pub easing: EasingCurve,
    pub elapsed_secs: f32,
    pub is_finished: bool,
}

impl KeyframeAnimation {
    pub fn new(start: f32, end: f32, duration_secs: f32, easing: EasingCurve) -> Self {
        Self {
            start_value: start,
            end_value: end,
            duration_secs: duration_secs.max(0.001),
            easing,
            elapsed_secs: 0.0,
            is_finished: false,
        }
    }

    /// Steps animation forward and returns current interpolated value.
    pub fn update(&mut self, dt_secs: f32) -> f32 {
        self.elapsed_secs += dt_secs;
        let progress = (self.elapsed_secs / self.duration_secs).clamp(0.0, 1.0);
        if progress >= 1.0 {
            self.is_finished = true;
        }

        let eased = self.easing.evaluate(progress);
        self.start_value + (self.end_value - self.start_value) * eased
    }

    pub fn reset(&mut self) {
        self.elapsed_secs = 0.0;
        self.is_finished = false;
    }
}

/// Physically-based Spring Physics Engine for dynamic widget animations.
#[derive(Debug, Clone)]
pub struct SpringPhysics {
    pub current_value: f32,
    pub target_value: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl SpringPhysics {
    pub fn new(initial: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            current_value: initial,
            target_value: initial,
            velocity: 0.0,
            stiffness,
            damping,
            mass: 1.0,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target_value = target;
    }

    /// Step simulation forward by delta time `dt_seconds`
    pub fn update(&mut self, dt_seconds: f32) -> f32 {
        let displacement = self.current_value - self.target_value;
        let spring_force = -self.stiffness * displacement;
        let damping_force = -self.damping * self.velocity;
        let acceleration = (spring_force + damping_force) / self.mass;

        self.velocity += acceleration * dt_seconds;
        self.current_value += self.velocity * dt_seconds;

        self.current_value
    }
}

/// Timeline animation scheduler coordinating multiple animations.
#[derive(Debug, Default)]
pub struct AnimationTimeline {
    animations: Vec<KeyframeAnimation>,
}

impl AnimationTimeline {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    pub fn add_animation(&mut self, anim: KeyframeAnimation) {
        self.animations.push(anim);
    }

    /// Steps all active timeline animations forward.
    pub fn update(&mut self, dt_secs: f32) {
        for anim in self.animations.iter_mut() {
            if !anim.is_finished {
                anim.update(dt_secs);
            }
        }
    }

    pub fn is_all_finished(&self) -> bool {
        self.animations.iter().all(|a| a.is_finished)
    }

    pub fn active_count(&self) -> usize {
        self.animations.iter().filter(|a| !a.is_finished).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_physics_convergence() {
        let mut spring = SpringPhysics::new(0.0, 180.0, 12.0);
        spring.set_target(100.0);

        for _ in 0..100 {
            spring.update(0.016);
        }

        assert!((spring.current_value - 100.0).abs() < 5.0);
    }

    #[test]
    fn test_easing_curves_boundary_conditions() {
        let curves = [
            EasingCurve::Linear,
            EasingCurve::EaseInQuad,
            EasingCurve::EaseOutQuad,
            EasingCurve::EaseInOutQuad,
            EasingCurve::EaseInCubic,
            EasingCurve::EaseOutCubic,
            EasingCurve::EaseInOutCubic,
            EasingCurve::EaseOutElastic,
            EasingCurve::EaseOutBounce,
        ];

        for curve in &curves {
            let start = curve.evaluate(0.0);
            let end = curve.evaluate(1.0);
            assert!((start - 0.0).abs() < 0.01, "{:?} start boundary failed", curve);
            assert!((end - 1.0).abs() < 0.01, "{:?} end boundary failed", curve);
        }
    }

    #[test]
    fn test_keyframe_animation_progress() {
        let mut anim = KeyframeAnimation::new(0.0, 200.0, 1.0, EasingCurve::EaseOutQuad);
        assert_eq!(anim.elapsed_secs, 0.0);
        assert!(!anim.is_finished);

        let mid = anim.update(0.5);
        assert!(mid > 0.0 && mid < 200.0);
        assert!(!anim.is_finished);

        let end = anim.update(0.6); // Total 1.1s >= 1.0s
        assert_eq!(end, 200.0);
        assert!(anim.is_finished);
    }

    #[test]
    fn test_animation_timeline_scheduling() {
        let mut timeline = AnimationTimeline::new();
        timeline.add_animation(KeyframeAnimation::new(0.0, 100.0, 0.5, EasingCurve::Linear));
        timeline.add_animation(KeyframeAnimation::new(0.0, 50.0, 1.0, EasingCurve::Linear));

        assert_eq!(timeline.active_count(), 2);
        assert!(!timeline.is_all_finished());

        timeline.update(0.6); // 1st finishes, 2nd still active
        assert_eq!(timeline.active_count(), 1);
        assert!(!timeline.is_all_finished());

        timeline.update(0.5); // 2nd finishes
        assert_eq!(timeline.active_count(), 0);
        assert!(timeline.is_all_finished());
    }
}
