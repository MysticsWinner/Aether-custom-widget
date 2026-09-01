//! Hardware Direct2D Physics Particle Simulation Engine
//!
//! Provides high-performance 144Hz physics particle simulation (Rain, Snow, Fog, Solar Rays, Lightning)
//! for ambient weather desktop widgets and dynamic visual effects.

pub mod emitter;
pub mod physics;

pub use emitter::{Particle, ParticleEmitter, ParticleType};
pub use physics::{ParticleFieldConfig, ParticlePhysicsEngine};
