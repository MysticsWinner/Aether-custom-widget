use serde::{Deserialize, Serialize};

/// Performance diagnostic recommendation from AI advisor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceRecommendation {
    pub widget_id: String,
    pub issue_type: String,
    pub suggestion: String,
    pub suggested_fps: u32,
}

/// AI Engine diagnosing resource bottlenecks and offering layout & frequency tuning.
pub struct AiPerformanceAdvisor;

impl AiPerformanceAdvisor {
    pub fn analyze(
        widget_id: &str,
        cpu_pct: f32,
        memory_mb: f32,
        tick_duration_us: u64,
    ) -> Vec<PerformanceRecommendation> {
        let mut recs = Vec::new();

        if cpu_pct > 15.0 || tick_duration_us > 1000 {
            recs.push(PerformanceRecommendation {
                widget_id: widget_id.to_string(),
                issue_type: "HighCpuUsage".to_string(),
                suggestion: format!(
                    "Widget '{}' uses {:.1}% CPU with {}us tick duration. Consider lowering target FPS from 60Hz to 15Hz.",
                    widget_id, cpu_pct, tick_duration_us
                ),
                suggested_fps: 15,
            });
        }

        if memory_mb > 50.0 {
            recs.push(PerformanceRecommendation {
                widget_id: widget_id.to_string(),
                issue_type: "MemoryFootprint".to_string(),
                suggestion: format!(
                    "Widget '{}' allocates {:.1}MB RAM. Consider enabling LRU texture caching.",
                    widget_id, memory_mb
                ),
                suggested_fps: 30,
            });
        }

        recs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_performance_advisor_diagnostics() {
        let recs = AiPerformanceAdvisor::analyze("clock_w", 25.0, 60.0, 1500);
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].issue_type, "HighCpuUsage");
        assert_eq!(recs[1].issue_type, "MemoryFootprint");
    }
}
