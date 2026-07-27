use crate::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas};
use std::time::Instant;
use tracing::info;

/// Performance benchmark harness evaluating Widget SDK draw batching throughput.
pub struct SdkBenchmark;

impl SdkBenchmark {
    pub fn run_benchmark() {
        let mut canvas = BatchRenderCanvas::new();
        let command_count = 100_000usize;

        let start = Instant::now();
        for _ in 0..command_count {
            canvas.draw_rect(
                RectF::new(10.0, 10.0, 100.0, 50.0),
                Color::rgba(1.0, 0.0, 0.0, 1.0),
                4.0,
            );
        }
        let elapsed = start.elapsed();

        let throughput_per_sec = (command_count as f64) / elapsed.as_secs_f64();
        info!(
            "SDK Benchmark: Batch Canvas Throughput = {:.0} draw commands / sec ({:?})",
            throughput_per_sec, elapsed
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_benchmark_execution() {
        SdkBenchmark::run_benchmark();
    }
}
