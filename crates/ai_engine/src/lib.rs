//! Next-Gen Windows Desktop Customization Platform - AI Engine Subsystem Crate
//!
//! Provides intelligent desktop customization across 6 pillars:
//! Desktop Automation, Voice Processing, AI Layout Generation, AI Theme Generation,
//! AI Widget Generation, and Workflow Automation.

pub mod benchmark;
pub mod composer;
pub mod generators;
pub mod performance_advisor;
pub mod voice;
pub mod wallpaper_theme;
pub mod widget_synthesizer;
pub mod workflow;

pub use benchmark::AiEngineBenchmark;
pub use composer::{AiDesktopComposer, ComposerOutput};
pub use generators::{LayoutGenerator, ThemeGenerator, WidgetGenerator};
pub use performance_advisor::{AiPerformanceAdvisor, PerformanceRecommendation};
pub use voice::VoiceIntentParser;
pub use wallpaper_theme::{WallpaperPalette, WallpaperThemeGenerator};
pub use widget_synthesizer::{SynthesizedWidget, WidgetSynthesizer};
pub use workflow::{WorkflowAutomationEngine, WorkflowRule};
