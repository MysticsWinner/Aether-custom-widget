# Aether Benchmarking Guide & Standard Operating Procedures

## Overview

This guide details the standard operating procedures for executing resource profiling and performance auditing across the **Aether Platform**.

---

## Benchmark Suite Documents

- **[CPU Idle Performance](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/benchmarks/cpu_idle.md)**: `< 0.1%` idle CPU usage.
- **[Memory Footprint](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/benchmarks/memory.md)**: `< 25 MB` total physical RAM working set.
- **[Startup Latency](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/benchmarks/startup.md)**: `< 45 ms` cold boot readiness.
- **[Refresh Rate & FPS](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/benchmarks/fps.md)**: 144 Hz+ DirectComposition hardware rendering.
- **[100-Widget Stress & Isolation](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/benchmarks/plugin_stress.md)**: Concurrency and AppContainer crash recovery.
- **[Aether vs Rainmeter Matrix](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/benchmarks/comparison_vs_rainmeter.md)**: Side-by-side technical comparison.

---

## Running Benchmarks Locally

```bash
# Run production engine profiler suite
cargo run --release -p production_engine -- --bench
```
