use crate::installer::PackageManager;
use std::time::Instant;
use tracing::info;

/// Benchmark harness measuring package installation & signature verification throughput.
pub struct PackageManagerBenchmark;

impl PackageManagerBenchmark {
    pub fn run_benchmark() {
        let mut pm = PackageManager::new();
        let count = 100usize;

        let start = Instant::now();
        for _ in 0..count {
            let _ = pm.install("weather-widget");
            let _ = pm.install("spotify-widget");
            let _ = pm.install("taskbar-plus");
        }
        let elapsed = start.elapsed();

        let total_installs = count * 3;
        let avg_install_ms = (elapsed.as_secs_f64() * 1000.0) / total_installs as f64;

        info!(
            "Package Manager Benchmark: {} Package Installs = {:?} ({:.3} ms / package install)",
            total_installs, elapsed, avg_install_ms
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_benchmark_execution() {
        PackageManagerBenchmark::run_benchmark();
    }
}
