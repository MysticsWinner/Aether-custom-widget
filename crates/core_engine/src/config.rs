use std::time::Duration;

/// Configuration parameters for the Core Engine Daemon.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Number of worker threads allocated for Tokio runtime (0 = auto / logical CPU count).
    pub thread_pool_size: usize,
    /// High-precision engine tick interval in milliseconds.
    pub tick_interval_ms: u64,
    /// Capacity of the multi-threaded CoreEvent broadcast channel buffer.
    pub event_channel_capacity: usize,
    /// Flag indicating whether system performance telemetry collection is enabled.
    pub telemetry_enabled: bool,
    /// Maximum allowed registered subsystems.
    pub max_subsystems: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: 0,
            tick_interval_ms: 10, // 10ms default tick (~100 Hz core update loop)
            event_channel_capacity: 1024,
            telemetry_enabled: true,
            max_subsystems: 64,
        }
    }
}

impl EngineConfig {
    /// Creates a new `EngineConfig` builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the thread pool size for background tasks.
    pub fn with_thread_pool_size(mut self, size: usize) -> Self {
        self.thread_pool_size = size;
        self
    }

    /// Sets the engine tick interval in milliseconds.
    pub fn with_tick_interval_ms(mut self, interval_ms: u64) -> Self {
        self.tick_interval_ms = interval_ms;
        self
    }

    /// Sets the event broadcast channel buffer capacity.
    pub fn with_event_channel_capacity(mut self, capacity: usize) -> Self {
        self.event_channel_capacity = capacity;
        self
    }

    /// Enables or disables telemetry collection.
    pub fn with_telemetry(mut self, enabled: bool) -> Self {
        self.telemetry_enabled = enabled;
        self
    }

    /// Returns the tick interval as a `Duration`.
    pub fn tick_duration(&self) -> Duration {
        Duration::from_millis(self.tick_interval_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.tick_interval_ms, 10);
        assert_eq!(config.event_channel_capacity, 1024);
        assert!(config.telemetry_enabled);
        assert_eq!(config.tick_duration(), Duration::from_millis(10));
    }

    #[test]
    fn test_builder_pattern() {
        let config = EngineConfig::new()
            .with_thread_pool_size(4)
            .with_tick_interval_ms(16)
            .with_event_channel_capacity(2048)
            .with_telemetry(false);

        assert_eq!(config.thread_pool_size, 4);
        assert_eq!(config.tick_interval_ms, 16);
        assert_eq!(config.event_channel_capacity, 2048);
        assert!(!config.telemetry_enabled);
    }
}
