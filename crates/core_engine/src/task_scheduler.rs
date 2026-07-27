use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{debug, info};

pub type ScheduledTask = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

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
}
