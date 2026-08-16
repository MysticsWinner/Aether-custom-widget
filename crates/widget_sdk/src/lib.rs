//! Next-Gen Windows Desktop Customization Platform - Master Widget SDK
//!
//! Standardized, multi-language API surface for 3rd-party widget development
//! providing 6 core pillars: Lifecycle, Rendering, Settings, Events, Animations, and Resources.

pub mod animations;
pub mod benchmark;
pub mod config;
pub mod contrast;
pub mod display_target;
pub mod events;
pub mod frame_scheduler;
pub mod lifecycle;
pub mod perf_budget;
pub mod reactive;
pub mod refresh_rate;
pub mod render_config;
pub mod rendering;
pub mod resource_cache;
pub mod resources;
pub mod settings;

pub use animations::{EasingCurve, SpringAnimation, SpringParams};
pub use config::{ColourOverrides, DisplayOptions, SwapMode, WidgetConfig};
pub use benchmark::SdkBenchmark;
pub use contrast::ContrastGuard;
pub use display_target::{DesktopLayer, DisplayTarget};
pub use events::{EventSubscriber, InputEvent, WidgetEvent};
pub use frame_scheduler::{FrameScheduler, WidgetFrameBudget};
pub use lifecycle::{TickContext, WidgetLifecycle, WidgetState};
pub use perf_budget::{BudgetEvaluator, BudgetState, PerformanceBudget};
pub use reactive::{Signal, SignalBinding};
pub use refresh_rate::{AdaptiveRefreshScheduler, UpdateFrequency};
pub use render_config::RenderConfig;
pub use rendering::{BatchRenderCanvas, Color, DrawCommand, RectF, RenderCanvas};
pub use resource_cache::LruResourceCache;
pub use resources::{InMemoryResourceManager, ResourceManager};
pub use settings::{InMemorySettingsStore, SettingValue, SettingsStore};
