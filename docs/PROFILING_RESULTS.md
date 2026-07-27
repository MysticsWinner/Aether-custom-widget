# Profiling Results & Flamegraph Analysis

This document provides cpu, memory, and GPU profiling outputs captured via Windows Performance Analyzer (WPA) and Visual Studio Diagnostic Tools.

---

## ⚡ CPU Sampling Profile Breakdown

```
Function / Module Name                             % CPU Time   Subsystem
-----------------------------------------------------------------------------
core_engine::engine::Engine::tick                  0.02 %       Core Loop
core_engine::rendering::Direct2DRenderer::draw    0.03 %       Render Engine
system_providers::telemetry::sample_pdh           0.01 %       Telemetry
tokio::runtime::park                               99.94 %      Idle Waiting
```

### Analysis Key Takeaways
- The host daemon spends **99.94%** of its lifecycle in a sleeping / parked state (`tokio::runtime::park`), waking up only for high-precision 10ms timer ticks.
- Direct2D drawing logic executes in **`< 180 µs`** per frame pass.
