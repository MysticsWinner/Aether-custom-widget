//! Next-Gen Windows Desktop Customization Platform - Master Widget SDK
//!
//! Standardized, multi-language API surface for 3rd-party widget development
//! providing 6 core pillars: Lifecycle, Rendering, Settings, Events, Animations, and Resources.

pub mod animations;
pub mod benchmark;
pub mod events;
pub mod lifecycle;
pub mod rendering;
pub mod resources;
pub mod settings;

pub use animations::{EasingCurve, SpringAnimation, SpringParams};
pub use benchmark::SdkBenchmark;
pub use events::{EventSubscriber, InputEvent, WidgetEvent};
pub use lifecycle::{TickContext, WidgetLifecycle, WidgetState};
pub use rendering::{BatchRenderCanvas, Color, DrawCommand, RectF, RenderCanvas};
pub use resources::{InMemoryResourceManager, ResourceManager};
pub use settings::{InMemorySettingsStore, SettingValue, SettingsStore};
