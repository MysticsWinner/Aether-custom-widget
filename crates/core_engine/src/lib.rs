//! Next-Gen Windows Desktop Customization Platform - Core Engine Library
//!
//! Provides the event-driven, multi-threaded, native host daemon infrastructure
//! responsible for system event routing, task scheduling, lifecycle management,
//! modular subsystem coordination, DirectComposition/Direct2D GPU rendering,
//! Data Engine telemetry streaming, Hot Reloadable Theme Engine integration,
//! Plugin Sandbox crash fault isolation, 13-Metric Performance Profiling,
//! Marketplace Package Manager, End-to-End Encrypted Cloud Sync Engine,
//! AI Intelligence Subsystem, Production Readiness Engineering, Failure Injection & ETW Tracing.

pub mod ai_subsystem;
pub mod cloud_subsystem;
pub mod config;
pub mod engine;
pub mod event_bus;
pub mod fault_diagnostics;
pub mod marketplace_subsystem;
pub mod plugin_subsystem;
pub mod production_subsystem;
pub mod profiler;
pub mod profiler_subsystem;
pub mod rendering;
pub mod subsystems;
pub mod task_scheduler;
pub mod telemetry_subsystem;
pub mod theme_subsystem;

pub use ai_subsystem::AiSubsystem;
pub use cloud_subsystem::CloudSyncSubsystem;
pub use config::EngineConfig;
pub use engine::{Engine, EngineState};
pub use event_bus::{CoreEvent, EventBus, EventPublisher};
pub use fault_diagnostics::{EtwTracingProvider, FailureInjector, FailurePoint, RedundancySupervisor};
pub use marketplace_subsystem::MarketplaceSubsystem;
pub use plugin_subsystem::PluginSandboxSubsystem;
pub use production_subsystem::ProductionSubsystem;
pub use profiler::{MasterPerformanceSuite, PerformanceProfileReport, SystemProfiler};
pub use profiler_subsystem::ProfilerSubsystem;
pub use rendering::{
    Color, Direct2DRenderer, DirtyRegionTracker, FrameStats, GpuRenderer, RainmeterBenchmark,
    RectF, RefreshRate, RenderBenchmarkResult,
};
pub use subsystems::{RenderSubsystem, Subsystem, SubsystemHealth, SubsystemManager};
pub use task_scheduler::TaskScheduler;
pub use telemetry_subsystem::TelemetrySubsystem;
pub use theme_subsystem::ThemeEngineSubsystem;
