use crate::generators::{LayoutGenerator, ThemeGenerator, WidgetGenerator};
use std::time::Instant;
use tracing::info;

/// Benchmark harness measuring AI generation and schema validation throughput.
pub struct AiEngineBenchmark;

impl AiEngineBenchmark {
    pub fn run_benchmark() {
        let count = 100usize;
        let start = Instant::now();

        for _ in 0..count {
            let _ = LayoutGenerator::generate_layout("4k desktop");
            let _ = ThemeGenerator::generate_theme("cyberpunk neon");
            let _ = WidgetGenerator::generate_widget("CPU monitor");
        }
        let elapsed = start.elapsed();

        let total_gens = count * 3;
        let avg_gen_ms = (elapsed.as_secs_f64() * 1000.0) / total_gens as f64;

        info!(
            "AI Engine Benchmark: {} AI Generation Passes = {:?} ({:.3} ms / AI generation pass)",
            total_gens, elapsed, avg_gen_ms
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_engine_benchmark_execution() {
        AiEngineBenchmark::run_benchmark();
    }
}
