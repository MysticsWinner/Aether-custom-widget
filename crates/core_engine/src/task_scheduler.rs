use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{debug, info};

pub type ScheduledTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Engine power profiles determining adaptive tick rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerProfile {
    /// 144 Hz / ~6.9 ms — during active user pointer interaction, dragging, audio FFT rendering
    HighPerformance,
    /// 60 Hz / ~16.6 ms — normal desktop rendering and active animations
    Standard,
    /// 15 Hz / ~66.6 ms — battery saver mode or unfocused background state
    PowerSaver,
    /// 1 Hz / 1000 ms — desktop idle > 60s or monitor standby
    UltraIdle,
    /// 0 Hz / Paused — full-screen exclusive 3D game active
    Suppressed,
}

impl PowerProfile {
    /// Returns the target refresh rate in Hertz.
    pub fn target_refresh_rate_hz(&self) -> u32 {
        match self {
            Self::HighPerformance => 144,
            Self::Standard => 60,
            Self::PowerSaver => 15,
            Self::UltraIdle => 1,
            Self::Suppressed => 0,
        }
    }

    /// Returns the target tick period duration.
    pub fn target_interval(&self) -> Duration {
        match self {
            Self::HighPerformance => Duration::from_micros(6944),
            Self::Standard => Duration::from_micros(16666),
            Self::PowerSaver => Duration::from_millis(66),
            Self::UltraIdle => Duration::from_secs(1),
            Self::Suppressed => Duration::from_secs(3600), // Dormant
        }
    }
}

/// Adaptive Power Governor — modulates engine tick rates and GPU redraw cycles
/// based on real-time user interaction, audio playback, and power topology.
#[derive(Debug, Clone)]
pub struct AdaptivePowerGovernor {
    current_profile: PowerProfile,
    total_ticks_modulated: u64,
}

impl AdaptivePowerGovernor {
    /// Creates a new `AdaptivePowerGovernor` initialized to `Standard` (60 Hz).
    pub fn new() -> Self {
        Self {
            current_profile: PowerProfile::Standard,
            total_ticks_modulated: 0,
        }
    }

    /// Evaluates system states and computes the optimal power profile.
    pub fn evaluate_profile(
        &mut self,
        is_interactive: bool,
        has_audio: bool,
        is_battery: bool,
        is_fullscreen_3d: bool,
        idle_seconds: u64,
    ) -> PowerProfile {
        self.total_ticks_modulated += 1;

        let new_profile = if is_fullscreen_3d {
            PowerProfile::Suppressed
        } else if is_interactive || has_audio {
            PowerProfile::HighPerformance
        } else if is_battery {
            PowerProfile::PowerSaver
        } else if idle_seconds >= 60 {
            PowerProfile::UltraIdle
        } else {
            PowerProfile::Standard
        };

        if self.current_profile != new_profile {
            info!(
                "AdaptivePowerGovernor switched profile: {:?} -> {:?} (Target: {} Hz)",
                self.current_profile,
                new_profile,
                new_profile.target_refresh_rate_hz()
            );
            self.current_profile = new_profile;
        }

        self.current_profile
    }

    /// Returns the current power profile.
    pub fn current_profile(&self) -> PowerProfile {
        self.current_profile
    }

    /// Estimates CPU energy savings percentage relative to unthrottled 144Hz.
    pub fn cpu_savings_estimate_pct(&self) -> f32 {
        match self.current_profile {
            PowerProfile::HighPerformance => 0.0,
            PowerProfile::Standard => 58.3,
            PowerProfile::PowerSaver => 89.6,
            PowerProfile::UltraIdle => 99.3,
            PowerProfile::Suppressed => 100.0,
        }
    }
}

impl Default for AdaptivePowerGovernor {
    fn default() -> Self {
        Self::new()
    }
}

/// High-precision multi-threaded Task Scheduler for executing periodic and async tasks.
pub struct TaskScheduler {
    tasks: Vec<JoinHandle<()>>,
}

impl TaskScheduler {
    /// Creates a new `TaskScheduler`.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Schedules a recurring periodic task executed at `period` interval.
    pub fn schedule_periodic<F, Fut>(&mut self, period: Duration, mut task_fn: F)
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        debug!("Scheduling periodic task with period {:?}", period);
        let handle = tokio::spawn(async move {
            let mut ticker = interval(period);
            loop {
                ticker.tick().await;
                task_fn().await;
            }
        });
        self.tasks.push(handle);
    }

    /// Schedules a one-shot task to execute after a specified delay.
    pub fn schedule_delayed<F, Fut>(&mut self, delay: Duration, task_fn: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        debug!("Scheduling delayed task with delay {:?}", delay);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            task_fn().await;
        });
        self.tasks.push(handle);
    }

    /// Cancels all running scheduled tasks.
    pub fn cancel_all(&mut self) {
        info!("Canceling {} scheduled tasks...", self.tasks.len());
        for handle in self.tasks.drain(..) {
            handle.abort();
        }
    }

    /// Returns the number of currently active scheduled tasks.
    pub fn active_task_count(&self) -> usize {
        self.tasks.iter().filter(|h| !h.is_finished()).count()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TaskScheduler {
    fn drop(&mut self) {
        self.cancel_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_periodic_scheduler() {
        let mut scheduler = TaskScheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        scheduler.schedule_periodic(Duration::from_millis(15), move || {
            let c = counter_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });

        assert_eq!(scheduler.active_task_count(), 1);

        sleep(Duration::from_millis(60)).await;
        let count = counter.load(Ordering::SeqCst);
        assert!(count >= 3, "Expected at least 3 ticks, got {}", count);

        scheduler.cancel_all();
        assert_eq!(scheduler.active_task_count(), 0);
    }

    #[tokio::test]
    async fn test_delayed_scheduler() {
        let mut scheduler = TaskScheduler::new();
        let executed = Arc::new(AtomicUsize::new(0));

        let exec_clone = executed.clone();
        scheduler.schedule_delayed(Duration::from_millis(20), move || async move {
            exec_clone.store(1, Ordering::SeqCst);
        });

        assert_eq!(executed.load(Ordering::SeqCst), 0);
        sleep(Duration::from_millis(40)).await;
        assert_eq!(executed.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_power_profile_refresh_rates_and_intervals() {
        assert_eq!(PowerProfile::HighPerformance.target_refresh_rate_hz(), 144);
        assert_eq!(PowerProfile::Standard.target_refresh_rate_hz(), 60);
        assert_eq!(PowerProfile::PowerSaver.target_refresh_rate_hz(), 15);
        assert_eq!(PowerProfile::UltraIdle.target_refresh_rate_hz(), 1);
        assert_eq!(PowerProfile::Suppressed.target_refresh_rate_hz(), 0);
    }

    #[test]
    fn test_adaptive_power_governor_evaluations() {
        let mut governor = AdaptivePowerGovernor::new();
        assert_eq!(governor.current_profile(), PowerProfile::Standard);

        // 1. Interactive -> HighPerformance
        let p1 = governor.evaluate_profile(true, false, false, false, 0);
        assert_eq!(p1, PowerProfile::HighPerformance);
        assert_eq!(governor.cpu_savings_estimate_pct(), 0.0);

        // 2. Fullscreen 3D -> Suppressed
        let p2 = governor.evaluate_profile(false, false, false, true, 0);
        assert_eq!(p2, PowerProfile::Suppressed);
        assert_eq!(governor.cpu_savings_estimate_pct(), 100.0);

        // 3. Battery -> PowerSaver
        let p3 = governor.evaluate_profile(false, false, true, false, 0);
        assert_eq!(p3, PowerProfile::PowerSaver);
        assert!(governor.cpu_savings_estimate_pct() > 80.0);

        // 4. Idle > 60s -> UltraIdle
        let p4 = governor.evaluate_profile(false, false, false, false, 120);
        assert_eq!(p4, PowerProfile::UltraIdle);
        assert!(governor.cpu_savings_estimate_pct() > 95.0);
    }
}
