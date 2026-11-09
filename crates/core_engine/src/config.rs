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
    /// Flag indicating whether development mode is active.
    /// In dev mode, garbage collection runs on a faster cadence (15s) and hot-reloading triggers aggressive GC.
    pub dev_mode: bool,
    /// Flag indicating whether aggressive garbage collection (cache purging & working set trimming) is enabled.
    pub aggressive_gc: bool,
    /// High-precision garbage collection maintenance interval in seconds.
    pub gc_interval_secs: u64,
    /// Flag indicating whether background periodic garbage collection is enabled.
    pub auto_gc_enabled: bool,
    /// Flag indicating whether synchronous startup benchmark & audit suites should run on launch.
    pub run_benchmarks: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: 0,
            tick_interval_ms: 10, // 10ms default tick (~100 Hz core update loop)
            event_channel_capacity: 1024,
            telemetry_enabled: true,
            max_subsystems: 64,
            dev_mode: false,
            aggressive_gc: false,
            gc_interval_secs: 300, // 5 minutes default in production
            auto_gc_enabled: true,
            run_benchmarks: false,
        }
    }
}

impl EngineConfig {
    /// Creates a new `EngineConfig` builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an `EngineConfig` populated with defaults and overridden by environment flags
    /// (`AETHER_DEV_MODE`, `AETHER_AGGRESSIVE_GC`, `AETHER_GC_INTERVAL`).
    pub fn from_env() -> Self {
        Self::default().with_env_overrides()
    }

    /// Applies environment variable overrides for development mode and garbage collection flags.
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(val) = std::env::var("AETHER_DEV_MODE") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                self = self.with_dev_mode(true);
            }
        }
        if let Ok(val) = std::env::var("AETHER_AGGRESSIVE_GC") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                self = self.with_aggressive_gc(true);
            }
        }
        if let Ok(val) = std::env::var("AETHER_GC_INTERVAL") {
            if let Ok(secs) = val.parse::<u64>() {
                self = self.with_gc_interval_secs(secs);
            }
        }
        if let Ok(val) = std::env::var("AETHER_RUN_BENCHMARKS") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                self = self.with_run_benchmarks(true);
            }
        }
        self
    }

    /// Parses command-line arguments (`--benchmark`, `--dev`, `--aggressive-gc`, `--gc-interval <secs>`, `--disable-auto-gc`)
    /// and applies them to this `EngineConfig`.
    pub fn with_cli_args(mut self, args: &[String]) -> Self {
        let mut idx = 0;
        while idx < args.len() {
            match args[idx].as_str() {
                "--benchmark" | "--benchmarks" => {
                    self = self.with_run_benchmarks(true);
                }
                "--dev" | "--dev-mode" => {
                    self = self.with_dev_mode(true);
                }
                "--aggressive-gc" => {
                    self = self.with_aggressive_gc(true);
                }
                "--gc-interval" if idx + 1 < args.len() => {
                    if let Ok(val) = args[idx + 1].parse::<u64>() {
                        self = self.with_gc_interval_secs(val);
                    }
                    idx += 1;
                }
                "--disable-auto-gc" => {
                    self = self.with_auto_gc(false);
                }
                _ => {}
            }
            idx += 1;
        }
        self
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

    /// Sets the development mode flag. If enabled, defaults GC interval to 15 seconds if not explicitly set.
    pub fn with_dev_mode(mut self, dev_mode: bool) -> Self {
        self.dev_mode = dev_mode;
        if dev_mode && self.gc_interval_secs == 300 {
            self.gc_interval_secs = 15;
        }
        self
    }

    /// Sets the aggressive garbage collection flag.
    pub fn with_aggressive_gc(mut self, aggressive_gc: bool) -> Self {
        self.aggressive_gc = aggressive_gc;
        self
    }

    /// Sets the periodic garbage collection interval in seconds.
    pub fn with_gc_interval_secs(mut self, interval_secs: u64) -> Self {
        self.gc_interval_secs = interval_secs;
        self
    }

    /// Enables or disables background periodic garbage collection.
    pub fn with_auto_gc(mut self, enabled: bool) -> Self {
        self.auto_gc_enabled = enabled;
        self
    }

    /// Enables or disables synchronous startup benchmark execution.
    pub fn with_run_benchmarks(mut self, enabled: bool) -> Self {
        self.run_benchmarks = enabled;
        self
    }

    /// Returns the tick interval as a `Duration`.
    pub fn tick_duration(&self) -> Duration {
        Duration::from_millis(self.tick_interval_ms)
    }

    /// Returns the garbage collection interval as a `Duration`.
    pub fn gc_duration(&self) -> Duration {
        Duration::from_secs(self.gc_interval_secs)
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

    #[test]
    fn test_dev_mode_and_garbage_collection_flags() {
        let config = EngineConfig::new()
            .with_dev_mode(true)
            .with_aggressive_gc(true);

        assert!(config.dev_mode);
        assert!(config.aggressive_gc);
        // Dev mode automatically reduces default GC interval to 15s
        assert_eq!(config.gc_interval_secs, 15);
        assert_eq!(config.gc_duration(), Duration::from_secs(15));

        let custom_config = EngineConfig::new()
            .with_gc_interval_secs(60)
            .with_auto_gc(false);

        assert_eq!(custom_config.gc_interval_secs, 60);
        assert!(!custom_config.auto_gc_enabled);
    }

    #[test]
    fn test_core_engine_cli_flags_benchmark_opt_in() {
        let default_cfg = EngineConfig::default();
        assert!(!default_cfg.run_benchmarks);

        let args = vec![
            "core_engine".to_string(),
            "--benchmark".to_string(),
            "--dev".to_string(),
            "--aggressive-gc".to_string(),
            "--gc-interval".to_string(),
            "45".to_string(),
            "--disable-auto-gc".to_string(),
        ];
        let parsed = EngineConfig::default().with_cli_args(&args);
        assert!(parsed.run_benchmarks);
        assert!(parsed.dev_mode);
        assert!(parsed.aggressive_gc);
        assert_eq!(parsed.gc_interval_secs, 45);
        assert!(!parsed.auto_gc_enabled);
    }
}

