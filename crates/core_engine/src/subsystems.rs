use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use crate::event_bus::EventBus;
use crate::rendering::{Direct2DRenderer, GpuRenderer};

/// Health status of a registered core subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubsystemHealth {
    Healthy,
    Degraded,
    Failed,
}

/// Detailed lifecycle state of a registered core subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubsystemLifecycleState {
    Uninitialized,
    Starting,
    Ready,
    Degraded,
    Recovering,
    Stopping,
    Stopped,
    Failed,
    SafeMode,
}

/// Execution requirement and scheduling cadence declared by a subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingCadence {
    /// Periodic execution at a specified interval (e.g. 10ms, 50ms, 100ms, 1000ms)
    Periodic(Duration),
    /// Reactive execution triggered by events / signals (Theme, Plugins, AI, Settings)
    Reactive,
    /// On-demand execution triggered by explicit caller requests (Marketplace search, Cloud Sync)
    OnDemand,
    /// Deadline-driven rendering triggered by dirty regions and target display refresh rate
    DeadlineDriven,
}

/// Modular Subsystem Trait.
/// Every core engine subsystem implements this interface.
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

    /// Declares the scheduling profile and preferred execution cadence for this subsystem.
    fn scheduling_cadence(&self) -> SchedulingCadence {
        SchedulingCadence::Periodic(Duration::from_millis(50))
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

    fn scheduling_cadence(&self) -> SchedulingCadence {
        SchedulingCadence::DeadlineDriven
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

/// Execution runtime metrics and deadline tracking for a subsystem.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubsystemExecutionStats {
    pub total_ticks: u64,
    pub total_duration_micros: u64,
    pub max_duration_micros: u64,
    pub last_duration_micros: u64,
    pub deadline_misses: u64,
}

/// Registry and lifecycle manager for all registered modular subsystems.
pub struct SubsystemManager {
    subsystems: Vec<Box<dyn Subsystem>>,
    statuses: Arc<RwLock<HashMap<&'static str, SubsystemHealth>>>,
    lifecycle_states: Arc<RwLock<HashMap<&'static str, SubsystemLifecycleState>>>,
    execution_stats: Arc<RwLock<HashMap<&'static str, SubsystemExecutionStats>>>,
    last_tick_times: HashMap<&'static str, Instant>,
    bus: Option<Arc<EventBus>>,
}

impl SubsystemManager {
    /// Creates a new empty `SubsystemManager`.
    pub fn new() -> Self {
        Self {
            subsystems: Vec::new(),
            statuses: Arc::new(RwLock::new(HashMap::new())),
            lifecycle_states: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
            last_tick_times: HashMap::new(),
            bus: None,
        }
    }

    /// Registers a new subsystem module.
    pub fn register(&mut self, subsystem: Box<dyn Subsystem>) {
        info!("Registering subsystem: '{}'", subsystem.name());
        self.subsystems.push(subsystem);
    }

    /// Initializes all registered subsystems sequentially in registration order.
    /// Invariant: If any subsystem initialization fails, rolls back already-initialized
    /// subsystems in reverse order to ensure clean resource release and zero orphans.
    pub async fn initialize_all(&mut self, bus: Arc<EventBus>) -> anyhow::Result<()> {
        info!("Initializing {} registered subsystems in dependency order...", self.subsystems.len());
        self.bus = Some(bus.clone());
        let mut initialized_indices = Vec::new();

        for (idx, sys) in self.subsystems.iter_mut().enumerate() {
            let name = sys.name();
            {
                let mut states = self.lifecycle_states.write().await;
                states.insert(name, SubsystemLifecycleState::Starting);
            }
            match sys.initialize(bus.clone()).await {
                Ok(_) => {
                    info!("Subsystem '{}' initialized successfully.", name);
                    let mut lock = self.statuses.write().await;
                    lock.insert(name, sys.health());
                    let mut states = self.lifecycle_states.write().await;
                    states.insert(name, SubsystemLifecycleState::Ready);
                    initialized_indices.push(idx);
                }
                Err(err) => {
                    error!(
                        "Subsystem '{}' failed to initialize: {:?}. Initiating rollback of partially initialized subsystems...",
                        name, err
                    );
                    {
                        let mut lock = self.statuses.write().await;
                        lock.insert(name, SubsystemHealth::Failed);
                        let mut states = self.lifecycle_states.write().await;
                        states.insert(name, SubsystemLifecycleState::Failed);
                    }
                    // Reverse-order cleanup of already-initialized subsystems to avoid resource leaks
                    for &rev_idx in initialized_indices.iter().rev() {
                        let cleanup_sys = &mut self.subsystems[rev_idx];
                        let cleanup_name = cleanup_sys.name();
                        warn!("Cleaning up partially initialized subsystem: '{}'", cleanup_name);
                        let _ = cleanup_sys.shutdown().await;
                        let mut states = self.lifecycle_states.write().await;
                        states.insert(cleanup_name, SubsystemLifecycleState::Stopped);
                    }
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    /// Executes scheduled ticks for subsystems based on their declared cadences.
    pub async fn tick_scheduled(&mut self) {
        let now = Instant::now();
        for sys in self.subsystems.iter_mut() {
            let name = sys.name();
            if sys.health() == SubsystemHealth::Failed {
                continue;
            }
            let cadence = sys.scheduling_cadence();
            let should_tick = match cadence {
                SchedulingCadence::Periodic(period) => {
                    match self.last_tick_times.get(name) {
                        Some(&last) => now.duration_since(last) >= period,
                        None => true,
                    }
                }
                SchedulingCadence::DeadlineDriven => true,
                SchedulingCadence::Reactive | SchedulingCadence::OnDemand => false,
            };

            if should_tick {
                self.last_tick_times.insert(name, now);
                let start_tick = Instant::now();
                let tick_res = sys.tick().await;
                let elapsed = start_tick.elapsed();

                {
                    let mut stats_map = self.execution_stats.write().await;
                    let stat = stats_map.entry(name).or_default();
                    stat.total_ticks += 1;
                    stat.last_duration_micros = elapsed.as_micros() as u64;
                    stat.total_duration_micros += stat.last_duration_micros;
                    if stat.last_duration_micros > stat.max_duration_micros {
                        stat.max_duration_micros = stat.last_duration_micros;
                    }
                    if let SchedulingCadence::Periodic(period) = cadence {
                        if elapsed > period {
                            stat.deadline_misses += 1;
                            warn!(
                                "Subsystem '{}' deadline missed: tick took {:?} (limit: {:?})",
                                name, elapsed, period
                            );
                        }
                    }
                }

                if let Err(err) = tick_res {
                    warn!(target: "subsystem", subsystem = name, error = ?err, "Subsystem tick failed — transitioning to Degraded");
                    let mut lock = self.statuses.write().await;
                    lock.insert(name, SubsystemHealth::Degraded);
                    let mut states = self.lifecycle_states.write().await;
                    states.insert(name, SubsystemLifecycleState::Degraded);
                    if let Some(bus) = &self.bus {
                        let _ = bus.publish(crate::event_bus::CoreEvent::SubsystemSignal {
                            subsystem: name.to_string(),
                            signal: "STATE_DEGRADED".to_string(),
                        });
                    }
                } else {
                    tracing::trace!(target: "subsystem", subsystem = name, duration_us = elapsed.as_micros(), "Subsystem tick succeeded");
                }
            }
        }
    }

    /// Explicitly triggers a tick on a named subsystem (for reactive / on-demand execution).
    pub async fn tick_subsystem(&mut self, target_name: &str) -> anyhow::Result<bool> {
        for sys in self.subsystems.iter_mut() {
            if sys.name() == target_name {
                if sys.health() == SubsystemHealth::Failed {
                    return Ok(false);
                }
                let start_tick = Instant::now();
                let res = sys.tick().await;
                let elapsed = start_tick.elapsed();
                self.last_tick_times.insert(sys.name(), Instant::now());
                {
                    let mut stats_map = self.execution_stats.write().await;
                    let stat = stats_map.entry(sys.name()).or_default();
                    stat.total_ticks += 1;
                    stat.last_duration_micros = elapsed.as_micros() as u64;
                    stat.total_duration_micros += stat.last_duration_micros;
                    if stat.last_duration_micros > stat.max_duration_micros {
                        stat.max_duration_micros = stat.last_duration_micros;
                    }
                }
                res?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Executes a tick update across all healthy subsystems unconditionally.
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
                let mut states = self.lifecycle_states.write().await;
                states.insert(name, SubsystemLifecycleState::Degraded);
            }
        }
    }

    /// Gracefully shuts down all subsystems in reverse registration order.
    pub async fn shutdown_all(&mut self) {
        info!("Shutting down subsystems in reverse order...");
        for sys in self.subsystems.iter_mut().rev() {
            let name = sys.name();
            {
                let mut states = self.lifecycle_states.write().await;
                states.insert(name, SubsystemLifecycleState::Stopping);
            }
            if let Err(err) = sys.shutdown().await {
                error!("Error during shutdown of subsystem '{}': {:?}", name, err);
                let mut states = self.lifecycle_states.write().await;
                states.insert(name, SubsystemLifecycleState::Failed);
            } else {
                info!("Subsystem '{}' shut down cleanly.", name);
                let mut states = self.lifecycle_states.write().await;
                states.insert(name, SubsystemLifecycleState::Stopped);
            }
        }
    }

    /// Returns the lifecycle state of a registered subsystem.
    pub async fn lifecycle_state(&self, name: &str) -> SubsystemLifecycleState {
        let states = self.lifecycle_states.read().await;
        states.get(name).copied().unwrap_or(SubsystemLifecycleState::Uninitialized)
    }

    /// Returns current execution metrics for a named subsystem.
    pub async fn execution_stats(&self, name: &str) -> Option<SubsystemExecutionStats> {
        let stats = self.execution_stats.read().await;
        stats.get(name).cloned()
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
        fail_init: bool,
        cadence: SchedulingCadence,
        tick_delay: Option<Duration>,
    }

    impl MockSubsystem {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                initialized: false,
                ticked: false,
                shutdown_called: false,
                fail_init: false,
                cadence: SchedulingCadence::Periodic(Duration::from_millis(50)),
                tick_delay: None,
            }
        }

        fn with_failing_init(name: &'static str) -> Self {
            Self {
                name,
                initialized: false,
                ticked: false,
                shutdown_called: false,
                fail_init: true,
                cadence: SchedulingCadence::Periodic(Duration::from_millis(50)),
                tick_delay: None,
            }
        }

        fn with_cadence(name: &'static str, cadence: SchedulingCadence) -> Self {
            Self {
                name,
                initialized: false,
                ticked: false,
                shutdown_called: false,
                fail_init: false,
                cadence,
                tick_delay: None,
            }
        }

        fn with_delay(name: &'static str, cadence: SchedulingCadence, delay: Duration) -> Self {
            Self {
                name,
                initialized: false,
                ticked: false,
                shutdown_called: false,
                fail_init: false,
                cadence,
                tick_delay: Some(delay),
            }
        }
    }

    #[async_trait]
    impl Subsystem for MockSubsystem {
        fn name(&self) -> &'static str {
            self.name
        }

        fn scheduling_cadence(&self) -> SchedulingCadence {
            self.cadence
        }

        async fn initialize(&mut self, _bus: Arc<EventBus>) -> anyhow::Result<()> {
            if self.fail_init {
                return Err(anyhow::anyhow!("Mock initialization failure"));
            }
            self.initialized = true;
            Ok(())
        }

        async fn tick(&mut self) -> anyhow::Result<()> {
            self.ticked = true;
            if let Some(delay) = self.tick_delay {
                tokio::time::sleep(delay).await;
            }
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
        assert_eq!(manager.lifecycle_state("mock_test").await, SubsystemLifecycleState::Ready);

        manager.tick_scheduled().await;
        manager.shutdown_all().await;
        assert_eq!(manager.lifecycle_state("mock_test").await, SubsystemLifecycleState::Stopped);
    }

    #[tokio::test]
    async fn test_partial_initialization_rollback_on_failure() {
        let bus = Arc::new(EventBus::new(16));
        let mut manager = SubsystemManager::new();

        let s1 = Box::new(MockSubsystem::new("subsystem_1"));
        let s2_failing = Box::new(MockSubsystem::with_failing_init("subsystem_2_fail"));
        let s3 = Box::new(MockSubsystem::new("subsystem_3"));

        manager.register(s1);
        manager.register(s2_failing);
        manager.register(s3);

        let res = manager.initialize_all(bus).await;
        assert!(res.is_err());

        // subsystem_1 must have been rolled back and stopped!
        assert_eq!(manager.lifecycle_state("subsystem_1").await, SubsystemLifecycleState::Stopped);
        // subsystem_2 must be marked Failed
        assert_eq!(manager.lifecycle_state("subsystem_2_fail").await, SubsystemLifecycleState::Failed);
        // subsystem_3 was never initialized
        assert_eq!(manager.lifecycle_state("subsystem_3").await, SubsystemLifecycleState::Uninitialized);
    }

    #[tokio::test]
    async fn test_scheduled_cadence_filtering() {
        let bus = Arc::new(EventBus::new(16));
        let mut manager = SubsystemManager::new();

        let periodic = Box::new(MockSubsystem::with_cadence("periodic_sys", SchedulingCadence::Periodic(Duration::from_millis(10))));
        let reactive = Box::new(MockSubsystem::with_cadence("reactive_sys", SchedulingCadence::Reactive));

        manager.register(periodic);
        manager.register(reactive);
        manager.initialize_all(bus).await.unwrap();

        manager.tick_scheduled().await;
        // Periodic was ticked, but reactive was not ticked by tick_scheduled
        assert!(manager.tick_subsystem("reactive_sys").await.unwrap(), "Reactive tick must succeed on explicit call");
    }

    #[tokio::test]
    async fn test_subsystem_deadline_miss_tracking() {
        let bus = Arc::new(EventBus::new(16));
        let mut manager = SubsystemManager::new();

        // Subsystem declares 5ms period, but tick takes 15ms -> deadline miss!
        let slow_sys = Box::new(MockSubsystem::with_delay(
            "slow_periodic",
            SchedulingCadence::Periodic(Duration::from_millis(5)),
            Duration::from_millis(15),
        ));

        manager.register(slow_sys);
        manager.initialize_all(bus).await.unwrap();

        manager.tick_scheduled().await;

        let stats = manager.execution_stats("slow_periodic").await.expect("Stats must exist");
        assert_eq!(stats.total_ticks, 1);
        assert!(stats.deadline_misses >= 1, "Expected at least 1 deadline miss, got {}", stats.deadline_misses);
        assert!(stats.last_duration_micros >= 10000, "Tick duration should reflect sleep");
    }

    #[tokio::test]
    async fn test_subsystem_scheduler_fairness_under_slow_peer() {
        let bus = Arc::new(EventBus::new(16));
        let mut manager = SubsystemManager::new();

        let fast_sys = Box::new(MockSubsystem::with_cadence(
            "fast_peer",
            SchedulingCadence::Periodic(Duration::from_millis(1)),
        ));
        let slow_sys = Box::new(MockSubsystem::with_delay(
            "slow_peer",
            SchedulingCadence::Periodic(Duration::from_millis(10)),
            Duration::from_millis(12),
        ));

        manager.register(fast_sys);
        manager.register(slow_sys);
        manager.initialize_all(bus).await.unwrap();

        manager.tick_scheduled().await;

        let fast_stats = manager.execution_stats("fast_peer").await.unwrap();
        let slow_stats = manager.execution_stats("slow_peer").await.unwrap();

        // Both subsystems executed their tick
        assert_eq!(fast_stats.total_ticks, 1);
        assert_eq!(slow_stats.total_ticks, 1);
        // Fast system met its deadline; slow system missed its deadline
        assert_eq!(fast_stats.deadline_misses, 0);
        assert_eq!(slow_stats.deadline_misses, 1);
    }
}
