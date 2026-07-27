use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use crate::event_bus::EventBus;
use crate::rendering::{Direct2DRenderer, GpuRenderer};

/// Health status of a registered core subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsystemHealth {
    Healthy,
    Degraded,
    Failed,
}

/// Modular Subsystem Trait.
/// Every core engine subsystem (e.g. Render Engine, Theme Manager, System Provider, IPC, Layout) implements this interface.
#[async_trait]
pub trait Subsystem: Send + Sync {
    /// Returns the unique string identifier for the subsystem.
    fn name(&self) -> &'static str;

    /// Initializes the subsystem using the core `EventBus`.
    async fn initialize(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()>;

    /// Executes a periodic subsystem tick update.
    async fn tick(&mut self) -> anyhow::Result<()>;

    /// Gracefully shuts down the subsystem and releases held resources.
    async fn shutdown(&mut self) -> anyhow::Result<()>;

    /// Returns current health status of the subsystem.
    fn health(&self) -> SubsystemHealth {
        SubsystemHealth::Healthy
    }
}

/// Subsystem wrapper for the Phase 6 Direct2D / DirectComposition GPU Rendering Engine.
pub struct RenderSubsystem {
    renderer: Box<dyn GpuRenderer>,
}

impl RenderSubsystem {
    pub fn new() -> Self {
        Self {
            renderer: Box::new(Direct2DRenderer::new()),
        }
    }

    pub fn with_renderer(renderer: Box<dyn GpuRenderer>) -> Self {
        Self { renderer }
    }
}

impl Default for RenderSubsystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Subsystem for RenderSubsystem {
    fn name(&self) -> &'static str {
        "gpu_render_engine"
    }

    async fn initialize(&mut self, _bus: Arc<EventBus>) -> anyhow::Result<()> {
        self.renderer.initialize()
    }

    async fn tick(&mut self) -> anyhow::Result<()> {
        if self.renderer.begin_frame() {
            self.renderer.draw_dirty_regions()?;
            self.renderer.end_frame()?;
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        info!("RenderSubsystem shut down.");
        Ok(())
    }
}

/// Registry and lifecycle manager for all registered modular subsystems.
pub struct SubsystemManager {
    subsystems: Vec<Box<dyn Subsystem>>,
    statuses: Arc<RwLock<HashMap<&'static str, SubsystemHealth>>>,
}

impl SubsystemManager {
    /// Creates a new empty `SubsystemManager`.
    pub fn new() -> Self {
        Self {
            subsystems: Vec::new(),
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new subsystem module.
    pub fn register(&mut self, subsystem: Box<dyn Subsystem>) {
        info!("Registering subsystem: '{}'", subsystem.name());
        self.subsystems.push(subsystem);
    }

    /// Initializes all registered subsystems sequentially in registration order.
    pub async fn initialize_all(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing {} registered subsystems...", self.subsystems.len());
        for sys in self.subsystems.iter_mut() {
            let name = sys.name();
            match sys.initialize(bus.clone()).await {
                Ok(_) => {
                    info!("Subsystem '{}' initialized successfully.", name);
                    let mut lock = self.statuses.write().await;
                    lock.insert(name, sys.health());
                }
                Err(err) => {
                    error!("Subsystem '{}' failed to initialize: {:?}", name, err);
                    let mut lock = self.statuses.write().await;
                    lock.insert(name, SubsystemHealth::Failed);
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    /// Executes a tick update across all healthy subsystems.
    pub async fn tick_all(&mut self) {
        for sys in self.subsystems.iter_mut() {
            let name = sys.name();
            if sys.health() == SubsystemHealth::Failed {
                continue;
            }
            if let Err(err) = sys.tick().await {
                warn!("Error ticking subsystem '{}': {:?}", name, err);
                let mut lock = self.statuses.write().await;
                lock.insert(name, SubsystemHealth::Degraded);
            }
        }
    }

    /// Gracefully shuts down all subsystems in reverse registration order.
    pub async fn shutdown_all(&mut self) {
        info!("Shutting down subsystems...");
        for sys in self.subsystems.iter_mut().rev() {
            let name = sys.name();
            if let Err(err) = sys.shutdown().await {
                error!("Error during shutdown of subsystem '{}': {:?}", name, err);
            } else {
                info!("Subsystem '{}' shut down cleanly.", name);
            }
        }
    }

    /// Returns the number of registered subsystems.
    pub fn count(&self) -> usize {
        self.subsystems.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSubsystem {
        name: &'static str,
        initialized: bool,
        ticked: bool,
        shutdown_called: bool,
    }

    impl MockSubsystem {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                initialized: false,
                ticked: false,
                shutdown_called: false,
            }
        }
    }

    #[async_trait]
    impl Subsystem for MockSubsystem {
        fn name(&self) -> &'static str {
            self.name
        }

        async fn initialize(&mut self, _bus: Arc<EventBus>) -> anyhow::Result<()> {
            self.initialized = true;
            Ok(())
        }

        async fn tick(&mut self) -> anyhow::Result<()> {
            self.ticked = true;
            Ok(())
        }

        async fn shutdown(&mut self) -> anyhow::Result<()> {
            self.shutdown_called = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_subsystem_lifecycle() {
        let bus = Arc::new(EventBus::new(16));
        let mut manager = SubsystemManager::new();

        let mock = Box::new(MockSubsystem::new("mock_test"));
        manager.register(mock);

        let render_sys = Box::new(RenderSubsystem::new());
        manager.register(render_sys);

        assert_eq!(manager.count(), 2);

        manager.initialize_all(bus).await.unwrap();
        manager.tick_all().await;
        manager.shutdown_all().await;

        assert_eq!(manager.count(), 2);
    }
}
