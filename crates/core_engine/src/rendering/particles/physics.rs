//! Particle Physics & Collision Simulation
//!
//! Simulates gravity, wind vectors, Brownian motion, and bounding-box collision detection
//! against desktop widget boundaries.

use crate::rendering::particles::emitter::{Particle, ParticleType};
use crate::rendering::RectF;

/// Configuration parameters for physical environmental forces.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleFieldConfig {
    pub wind_velocity_x: f32,
    pub gravity_y: f32,
    pub turbulence: f32,
}

impl Default for ParticleFieldConfig {
    fn default() -> Self {
        Self {
            wind_velocity_x: -15.0,
            gravity_y: 980.0,
            turbulence: 1.0,
        }
    }
}

/// Physical particle simulation engine.
#[derive(Debug, Clone)]
pub struct ParticlePhysicsEngine {
    config: ParticleFieldConfig,
    viewport_width: f32,
    viewport_height: f32,
    total_collisions: u64,
}

impl ParticlePhysicsEngine {
    /// Creates a new `ParticlePhysicsEngine`.
    pub fn new(config: ParticleFieldConfig, width: f32, height: f32) -> Self {
        Self {
            config,
            viewport_width: width,
            viewport_height: height,
            total_collisions: 0,
        }
    }

    /// Advances particle simulation by `dt_secs`.
    pub fn step(
        &mut self,
        dt_secs: f32,
        particles: &mut Vec<Particle>,
        obstacles: &[RectF],
    ) {
        let mut splashes = Vec::new();

        for p in particles.iter_mut() {
            if !p.is_alive {
                continue;
            }

            // Apply lifetime decay
            p.lifetime_secs -= dt_secs;
            if p.lifetime_secs <= 0.0 {
                p.is_alive = false;
                continue;
            }

            // Apply physical forces
            match p.p_type {
                ParticleType::RainDrop => {
                    p.x += (p.vx + self.config.wind_velocity_x) * dt_secs;
                    p.y += p.vy * dt_secs;
                }
                ParticleType::SnowFlake => {
                    let drift = (p.lifetime_secs * 4.0).sin() * 20.0 * self.config.turbulence;
                    p.x += (p.vx + self.config.wind_velocity_x * 0.5 + drift) * dt_secs;
                    p.y += p.vy * dt_secs;
                }
                ParticleType::FogCloud => {
                    p.x += (p.vx + self.config.wind_velocity_x * 0.2) * dt_secs;
                    if p.x > self.viewport_width + 100.0 {
                        p.x = -100.0;
                    }
                }
                ParticleType::SplashDroplet => {
                    p.vy += self.config.gravity_y * dt_secs;
                    p.x += p.vx * dt_secs;
                    p.y += p.vy * dt_secs;
                }
                _ => {
                    p.x += p.vx * dt_secs;
                    p.y += p.vy * dt_secs;
                }
            }

            // Check collision against obstacle rectangles (e.g. widget cards)
            for obs in obstacles {
                if p.p_type == ParticleType::RainDrop
                    && p.x >= obs.x
                    && p.x <= obs.right()
                    && p.y >= obs.y
                    && p.y <= obs.y + 15.0
                {
                    p.is_alive = false;
                    self.total_collisions += 1;
                    // Queue splash droplets
                    splashes.push((p.x, obs.y));
                    break;
                }
            }

            // Viewport boundary cull
            if p.y > self.viewport_height + 50.0 || p.x < -100.0 || p.x > self.viewport_width + 100.0 {
                p.is_alive = false;
            }
        }

        // Retain only live particles
        particles.retain(|p| p.is_alive);

        // Spawn splash droplets
        for (sx, sy) in splashes {
            for i in 0..3 {
                let vx = -30.0 + (i as f32 * 30.0);
                particles.push(Particle::new(
                    ParticleType::SplashDroplet,
                    sx,
                    sy,
                    vx,
                    -70.0,
                    1.5,
                    0.8,
                    0.25,
                ));
            }
        }
    }

    /// Sets the live wind velocity vector.
    pub fn set_wind_velocity(&mut self, wind_x: f32) {
        self.config.wind_velocity_x = wind_x;
    }

    /// Returns total registered collisions.
    pub fn total_collisions(&self) -> u64 {
        self.total_collisions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_step_advances_particles() {
        let config = ParticleFieldConfig::default();
        let mut engine = ParticlePhysicsEngine::new(config, 800.0, 600.0);
        let mut particles = vec![Particle::new(
            ParticleType::RainDrop,
            100.0,
            100.0,
            0.0,
            500.0,
            2.0,
            1.0,
            2.0,
        )];

        engine.step(0.1, &mut particles, &[]);
        assert_eq!(particles.len(), 1);
        assert!(particles[0].y > 100.0, "Particle y should advance with positive vy");
    }

    #[test]
    fn test_physics_collision_with_obstacle() {
        let config = ParticleFieldConfig::default();
        let mut engine = ParticlePhysicsEngine::new(config, 800.0, 600.0);
        let mut particles = vec![Particle::new(
            ParticleType::RainDrop,
            150.0,
            195.0,
            0.0,
            200.0,
            2.0,
            1.0,
            2.0,
        )];

        let obstacle = RectF::new(100.0, 200.0, 200.0, 100.0);
        engine.step(0.05, &mut particles, &[obstacle]);

        assert!(engine.total_collisions() > 0, "Collision should be detected");
        // Splash droplets should have been created
        let splashes = particles.iter().filter(|p| p.p_type == ParticleType::SplashDroplet).count();
        assert!(splashes > 0, "Splashes should spawn on collision");
    }
}
