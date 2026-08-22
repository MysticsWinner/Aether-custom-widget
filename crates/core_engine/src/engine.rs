use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::EngineConfig;
use crate::event_bus::{CoreEvent, EventBus};
use crate::subsystems::{Subsystem, SubsystemManager};
use crate::task_scheduler::TaskScheduler;

/// Represents the current execution state of the Core Engine Daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Initializing,
    Running,
    Paused,
    Stopped,
}

/// Central Core Engine Host Daemon Coordinator.
pub struct Engine {
    config: EngineConfig,
    event_bus: Arc<EventBus>,
    subsystem_manager: SubsystemManager,
    task_scheduler: TaskScheduler,
    state: Arc<RwLock<EngineState>>,
}

impl Engine {
    /// Creates a new `Engine` instance with given configuration.
    pub fn new(config: EngineConfig) -> Self {
        let bus = Arc::new(EventBus::new(config.event_channel_capacity));
        Self {
            config,
            event_bus: bus,
            subsystem_manager: SubsystemManager::new(),
            task_scheduler: TaskScheduler::new(),
            state: Arc::new(RwLock::new(EngineState::Initializing)),
        }
    }

    /// Registers a modular subsystem into the engine.
    pub fn register_subsystem(&mut self, subsystem: Box<dyn Subsystem>) -> &mut Self {
        self.subsystem_manager.register(subsystem);
        self
    }

    /// Returns a reference to the shared `EventBus`.
    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    /// Returns the engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Returns the current state of the engine.
    pub async fn state(&self) -> EngineState {
        *self.state.read().await
    }

    /// Initializes all registered subsystems and starts the Core Engine loop.
    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting Core Engine Host Daemon...");

        // 1. Initialize Subsystems
        self.subsystem_manager.initialize_all(self.event_bus.clone()).await?;

        // 2. Set State to Running
        {
            let mut state = self.state.write().await;
            *state = EngineState::Running;
        }

        // 3. Emit State Event
        let _ = self.event_bus.publish(CoreEvent::SystemStateChanged {
            state: "Running".to_string(),
        });

        info!("Core Engine Daemon started and running successfully.");
        Ok(())
    }

    /// Executes a single tick step of the core engine.
    pub async fn tick(&mut self) {
        let current_state = self.state().await;
        if current_state != EngineState::Running {
            return;
        }

        // Tick all subsystems
        self.subsystem_manager.tick_all().await;
    }

    /// Pauses engine execution.
    pub async fn pause(&self) {
        let mut state = self.state.write().await;
        if *state == EngineState::Running {
            *state = EngineState::Paused;
            info!("Core Engine paused.");
            let _ = self.event_bus.publish(CoreEvent::SystemStateChanged {
                state: "Paused".to_string(),
            });
        }
    }

    /// Resumes engine execution from a paused state.
    pub async fn resume(&self) {
        let mut state = self.state.write().await;
        if *state == EngineState::Paused {
            *state = EngineState::Running;
            info!("Core Engine resumed.");
            let _ = self.event_bus.publish(CoreEvent::SystemStateChanged {
                state: "Running".to_string(),
            });
        }
    }

    /// Stops the Core Engine and cleans up resources.
    pub async fn stop(&mut self) {
        info!("Stopping Core Engine Daemon...");

        // 1. Cancel tasks
        self.task_scheduler.cancel_all();

        // 2. Shutdown Subsystems
        self.subsystem_manager.shutdown_all().await;

        // 3. Update State
        {
            let mut state = self.state.write().await;
            *state = EngineState::Stopped;
        }

        let _ = self.event_bus.publish(CoreEvent::SystemStateChanged {
            state: "Stopped".to_string(),
        });

        info!("Core Engine Daemon stopped cleanly.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_lifecycle() {
        let config = EngineConfig::default().with_tick_interval_ms(5);
        let mut engine = Engine::new(config);

        assert_eq!(engine.state().await, EngineState::Initializing);

        engine.start().await.unwrap();
        assert_eq!(engine.state().await, EngineState::Running);

        engine.tick().await;

        engine.pause().await;
        assert_eq!(engine.state().await, EngineState::Paused);

        engine.resume().await;
        assert_eq!(engine.state().await, EngineState::Running);

        engine.stop().await;
        assert_eq!(engine.state().await, EngineState::Stopped);
    }
}
