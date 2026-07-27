//! Next-Gen Windows Desktop Customization Platform - AI Engine Subsystem Crate
//!
//! Provides intelligent desktop customization across 6 pillars:
//! Desktop Automation, Voice Processing, AI Layout Generation, AI Theme Generation,
//! AI Widget Generation, and Workflow Automation.

pub mod benchmark;
pub mod generators;
pub mod voice;
pub mod workflow;

pub use benchmark::AiEngineBenchmark;
pub use generators::{LayoutGenerator, ThemeGenerator, WidgetGenerator};
pub use voice::VoiceIntentParser;
pub use workflow::{WorkflowAutomationEngine, WorkflowRule};
