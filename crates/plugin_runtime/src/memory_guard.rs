use serde::{Deserialize, Serialize};
use tracing::warn;

/// Status report issued by MemoryGuard evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUsageReport {
    pub widget_id: String,
    pub cpu_pct: f32,
    pub cpu_quota_pct: f32,
    pub memory_used_mb: f32,
    pub memory_limit_mb: f32,
    pub update_duration_ms: u64,
    pub update_budget_ms: u64,
    pub warning: Option<ResourceWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceWarning {
    CpuRunaway { duration_secs: u32 },
    MemoryLeak { usage_pct: u32 },
    UpdateBudgetExceeded { duration_ms: u64, budget_ms: u64 },
}

/// Proactive resource usage guard evaluating CPU, RAM, and tick budget thresholds.
pub struct MemoryGuard {
    cpu_warning_threshold_pct: f32,
    memory_warning_threshold_pct: f32,
}

impl MemoryGuard {
    pub fn new() -> Self {
        Self {
            cpu_warning_threshold_pct: 80.0,
            memory_warning_threshold_pct: 90.0,
        }
    }

    /// Evaluates current metrics against resource thresholds and emits a ResourceUsageReport.
    pub fn evaluate(
        &self,
        widget_id: &str,
        cpu_pct: f32,
        cpu_quota_pct: f32,
        memory_used_mb: f32,
        memory_limit_mb: f32,
        update_duration_ms: u64,
        update_budget_ms: u64,
    ) -> ResourceUsageReport {
        let memory_pct = (memory_used_mb / memory_limit_mb.max(1.0)) * 100.0;

        let warning = if cpu_pct > self.cpu_warning_threshold_pct {
            warn!(widget_id = %widget_id, cpu_pct, "MemoryGuard: CPU runaway threshold exceeded");
            Some(ResourceWarning::CpuRunaway { duration_secs: 5 })
        } else if memory_pct > self.memory_warning_threshold_pct {
            warn!(widget_id = %widget_id, memory_pct, "MemoryGuard: Memory leak threshold exceeded");
            Some(ResourceWarning::MemoryLeak {
                usage_pct: memory_pct as u32,
            })
        } else if update_duration_ms > update_budget_ms.saturating_mul(2) {
            warn!(widget_id = %widget_id, update_duration_ms, update_budget_ms, "MemoryGuard: Tick budget overrun");
            Some(ResourceWarning::UpdateBudgetExceeded {
                duration_ms: update_duration_ms,
                budget_ms: update_budget_ms,
            })
        } else {
            None
        };

        ResourceUsageReport {
            widget_id: widget_id.to_string(),
            cpu_pct,
            cpu_quota_pct,
            memory_used_mb,
            memory_limit_mb,
            update_duration_ms,
            update_budget_ms,
            warning,
        }
    }
}

impl Default for MemoryGuard {
    fn default() -> Self {
        Self::new()
    }
}
