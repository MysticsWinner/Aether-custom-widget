use std::collections::HashMap;

/// Per-widget frame budget tracking target FPS and rendering eligibility.
#[derive(Debug, Clone)]
pub struct WidgetFrameBudget {
    pub target_fps: u32,
    pub last_rendered_ms: u64,
}

/// Enforces per-widget render frequencies (e.g. 60Hz clock vs 10Hz gauge vs 0.1Hz weather).
#[derive(Debug, Clone, Default)]
pub struct FrameScheduler {
    budgets: HashMap<String, WidgetFrameBudget>,
}

impl FrameScheduler {
    pub fn new() -> Self {
        Self {
            budgets: HashMap::new(),
        }
    }

    pub fn set_widget_target_fps(&mut self, widget_id: &str, target_fps: u32) {
        let entry = self
            .budgets
            .entry(widget_id.to_string())
            .or_insert(WidgetFrameBudget {
                target_fps,
                last_rendered_ms: 0,
            });
        entry.target_fps = target_fps.max(1);
    }

    pub fn should_render(&mut self, widget_id: &str, now_ms: u64) -> bool {
        let budget = match self.budgets.get_mut(widget_id) {
            Some(b) => b,
            None => {
                // Default 60 FPS if unconfigured
                self.budgets.insert(
                    widget_id.to_string(),
                    WidgetFrameBudget {
                        target_fps: 60,
                        last_rendered_ms: 0,
                    },
                );
                self.budgets.get_mut(widget_id).unwrap()
            }
        };

        let min_interval_ms = 1000 / budget.target_fps.max(1) as u64;
        if budget.last_rendered_ms == 0 || now_ms.saturating_sub(budget.last_rendered_ms) >= min_interval_ms {
            budget.last_rendered_ms = now_ms;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_scheduler_enforces_target_fps() {
        let mut scheduler = FrameScheduler::new();
        // 10 FPS -> 100ms interval
        scheduler.set_widget_target_fps("gauge_widget", 10);

        // Render at t=1000ms -> true
        assert!(scheduler.should_render("gauge_widget", 1000));

        // Render at t=1050ms (< 100ms interval) -> false
        assert!(!scheduler.should_render("gauge_widget", 1050));

        // Render at t=1105ms (>= 100ms interval) -> true
        assert!(scheduler.should_render("gauge_widget", 1105));
    }
}
