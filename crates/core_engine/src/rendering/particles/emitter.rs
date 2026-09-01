//! Particle Emitter for Atmospheric Weather & Visual Effects
//!
//! Generates and recycles instanced particles for rain, snow, fog, solar rays, and lightning.

use serde::{Deserialize, Serialize};

/// Type of atmospheric particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleType {
    RainDrop,
    SnowFlake,
    FogCloud,
    SolarRay,
    LightningBolt,
    SplashDroplet,
}

/// A single simulated physical particle.
#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    pub p_type: ParticleType,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub opacity: f32,
    pub lifetime_secs: f32,
    pub max_lifetime_secs: f32,
    pub is_alive: bool,
}

impl Particle {
    pub fn new(p_type: ParticleType, x: f32, y: f32, vx: f32, vy: f32, size: f32, opacity: f32, lifetime: f32) -> Self {
        Self {
            p_type,
            x,
            y,
            vx,
            vy,
            size,
            opacity,
            lifetime_secs: lifetime,
            max_lifetime_secs: lifetime,
            is_alive: true,
        }
    }
}

/// Atmospheric Particle Emitter.
#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    p_type: ParticleType,
    max_particles: usize,
    emission_rate_per_sec: f32,
    emission_accumulator: f32,
    viewport_width: f32,
    viewport_height: f32,
    random_seed: u64,
}

impl ParticleEmitter {
    /// Creates a new `ParticleEmitter`.
    pub fn new(p_type: ParticleType, max_particles: usize, emission_rate: f32, width: f32, height: f32) -> Self {
        Self {
            p_type,
            max_particles,
            emission_rate_per_sec: emission_rate,
            emission_accumulator: 0.0,
            viewport_width: width,
            viewport_height: height,
            random_seed: 1337,
        }
    }

    /// Fast pseudo-random number generator (Xorshift64).
    fn next_rand(&mut self) -> f32 {
        self.random_seed ^= self.random_seed << 13;
        self.random_seed ^= self.random_seed >> 7;
        self.random_seed ^= self.random_seed << 17;
        (self.random_seed % 1000) as f32 / 1000.0
    }

    /// Emits new particles up to the maximum capacity.
    pub fn emit(&mut self, dt_secs: f32, pool: &mut Vec<Particle>) {
        self.emission_accumulator += self.emission_rate_per_sec * dt_secs;
        let count = self.emission_accumulator.floor() as usize;
        self.emission_accumulator -= count as f32;

        for _ in 0..count {
            if pool.len() >= self.max_particles {
                break;
            }

            let rx = self.next_rand() * self.viewport_width;
            let rsize = self.next_rand();
            let ropacity = self.next_rand();

            let p = match self.p_type {
                ParticleType::RainDrop => {
                    let speed = 600.0 + (rsize * 300.0);
                    Particle::new(
                        ParticleType::RainDrop,
                        rx,
                        -10.0,
                        -20.0 + (rsize * 10.0),
                        speed,
                        1.5 + (rsize * 1.5),
                        0.6 + (ropacity * 0.4),
                        2.5,
                    )
                }
                ParticleType::SnowFlake => {
                    let speed = 40.0 + (rsize * 50.0);
                    Particle::new(
                        ParticleType::SnowFlake,
                        rx,
                        -10.0,
                        -10.0 + (self.next_rand() * 20.0),
                        speed,
                        2.0 + (rsize * 4.0),
                        0.5 + (ropacity * 0.5),
                        8.0,
                    )
                }
                ParticleType::FogCloud => {
                    Particle::new(
                        ParticleType::FogCloud,
                        rx,
                        self.viewport_height * 0.5 + (self.next_rand() * (self.viewport_height * 0.5)),
                        15.0 + (rsize * 20.0),
                        0.0,
                        80.0 + (rsize * 120.0),
                        0.15 + (ropacity * 0.20),
                        15.0,
                    )
                }
                ParticleType::SolarRay => {
                    Particle::new(
                        ParticleType::SolarRay,
                        rx,
                        0.0,
                        10.0,
                        50.0,
                        40.0 + (rsize * 60.0),
                        0.2 + (ropacity * 0.3),
                        6.0,
                    )
                }
                ParticleType::LightningBolt => {
                    Particle::new(
                        ParticleType::LightningBolt,
                        rx,
                        0.0,
                        0.0,
                        0.0,
                        1.0,
                        1.0,
                        0.15,
                    )
                }
                ParticleType::SplashDroplet => {
                    Particle::new(
                        ParticleType::SplashDroplet,
                        rx,
                        self.viewport_height - 20.0,
                        -40.0 + (self.next_rand() * 80.0),
                        -80.0 - (self.next_rand() * 60.0),
                        1.5,
                        0.8,
                        0.35,
                    )
                }
            };
            pool.push(p);
        }
    }

    pub fn particle_type(&self) -> ParticleType {
        self.p_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_emitter_emits_rain() {
        let mut emitter = ParticleEmitter::new(ParticleType::RainDrop, 100, 50.0, 800.0, 600.0);
        let mut pool = Vec::new();

        emitter.emit(0.1, &mut pool);
        assert!(!pool.is_empty(), "Emitter should produce particles");
        assert_eq!(pool[0].p_type, ParticleType::RainDrop);
        assert!(pool[0].vy > 0.0, "Rain should move downwards");
    }

    #[test]
    fn test_particle_emitter_emits_snow() {
        let mut emitter = ParticleEmitter::new(ParticleType::SnowFlake, 50, 20.0, 800.0, 600.0);
        let mut pool = Vec::new();

        emitter.emit(0.2, &mut pool);
        assert!(!pool.is_empty());
        assert_eq!(pool[0].p_type, ParticleType::SnowFlake);
        assert!(pool[0].size >= 2.0);
    }
}
