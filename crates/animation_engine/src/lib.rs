/// Physically-based Spring Physics Engine for dynamic widget animations
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
}
