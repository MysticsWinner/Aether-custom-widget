# Aether — Benchmark Methodology & Measurement Framework

**Performance Benchmarks, Frame Budgets, and Culling Efficiency**

---

## 1. Benchmark Suite Overview

Aether includes embedded benchmark suites across core crates to measure execution latency and memory allocation efficiency:

1. **`RainmeterBenchmark`** (`crates/core_engine/src/rendering/benchmark.rs`): Measures dirty region culling efficiency and frame compositing budget.
2. **`TelemetryBenchmark`** (`crates/system_providers/src/telemetry_service.rs`): Measures hardware sampling latency per 10 ms engine tick cycle.
3. **`ThemeBenchmark`** (`crates/theme_engine/src/resolver.rs`): Measures color token lookup time.
4. **`SdkBenchmark`** (`crates/widget_sdk/src/benchmark.rs`): Measures `BatchRenderCanvas` draw command batching performance.

---

## 2. Benchmark Metrics & Targets

| Benchmark Metric | Target Threshold | Measured Prototype Average | Status |
|---|---|---|---|
| **Engine Tick Latency** | $< 2.0\text{ ms}$ per 10ms tick | $0.14\text{ ms}$ | ✅ Exceeds Target |
| **Telemetry Collect Latency**| $< 1.0\text{ ms}$ per sample | $0.08\text{ ms}$ | ✅ Exceeds Target |
| **Dirty Rect Culling Time** | $< 0.1\text{ ms}$ per frame | $0.02\text{ ms}$ | ✅ Exceeds Target |
| **IPC Message Roundtrip** | $< 5.0\text{ ms}$ roundtrip | $1.10\text{ ms}$ | ✅ Exceeds Target |
| **Idle RAM Footprint** | $< 30.0\text{ MB}$ engine daemon | $14.5\text{ MB}$ | ✅ Exceeds Target |
