//! Performance Budget & Adaptive Degradation System
//!
//! Tracks declared vs actual CPU %, RAM MB, FPS, material cost, and animation cost.
//! Enforces soft limits, warnings, adaptive degradation, and hard limits.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetState {
    Normal,
    SoftLimitExceeded,
    Warning,
    Degraded,
    HardLimitExceeded,
}

impl Default for BudgetState {
    fn default() -> Self {
        BudgetState::Normal
    }
}

/// Declarative visual & resource budget for a widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceBudget {
    pub target_cpu_pct: f32,
    pub target_memory_mb: f32,
    pub target_fps: u32,
    pub material_cost: String,   // "low", "medium", "high"
    pub animation_cost: String,  // "low", "medium", "high"
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        Self {
            target_cpu_pct: 0.10,
            target_memory_mb: 20.0,
            target_fps: 30,
            material_cost: "low".to_string(),
            animation_cost: "low".to_string(),
        }
    }
}

pub struct BudgetEvaluator;

impl BudgetEvaluator {
    /// Evaluates actual telemetry usage against declared performance budget.
    pub fn evaluate(budget: &PerformanceBudget, actual_cpu_pct: f32, actual_mem_mb: f32) -> BudgetState {
        let cpu_ratio = actual_cpu_pct / budget.target_cpu_pct.max(0.01);
        let mem_ratio = actual_mem_mb / budget.target_memory_mb.max(1.0);

        let max_ratio = cpu_ratio.max(mem_ratio);

        if max_ratio > 3.0 {
            BudgetState::HardLimitExceeded
        } else if max_ratio > 2.0 {
            BudgetState::Degraded
        } else if max_ratio > 1.5 {
            BudgetState::Warning
        } else if max_ratio > 1.0 {
            BudgetState::SoftLimitExceeded
        } else {
            BudgetState::Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_evaluator_states() {
        let budget = PerformanceBudget::default(); // target CPU 0.10%

        assert_eq!(BudgetEvaluator::evaluate(&budget, 0.05, 10.0), BudgetState::Normal);
        assert_eq!(BudgetEvaluator::evaluate(&budget, 0.12, 15.0), BudgetState::SoftLimitExceeded);
        assert_eq!(BudgetEvaluator::evaluate(&budget, 0.18, 15.0), BudgetState::Warning);
        assert_eq!(BudgetEvaluator::evaluate(&budget, 0.25, 15.0), BudgetState::Degraded);
        assert_eq!(BudgetEvaluator::evaluate(&budget, 0.40, 15.0), BudgetState::HardLimitExceeded);
    }
}
