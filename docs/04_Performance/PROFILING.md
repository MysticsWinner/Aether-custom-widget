# Aether — Profiling & Diagnostics Architecture

**Profiler Subsystem, ETW Tracing, and Latency Analysis**

---

## 1. `ProfilerSubsystem` Design

The `ProfilerSubsystem` (`crates/core_engine/src/profiler_subsystem.rs`) measures execution duration across engine ticks:

- **Metrics Tracked**: Tick interval variance, subsystem execution time, IPC command processing latency.
- **Diagnostics Output**: Exposed via `ControlCommand::GetDiagnostics` IPC endpoint.

---

## 2. Windows Event Tracing (ETW) Integration

The `EtwTracingProvider` inside `fault_diagnostics.rs` establishes high-frequency tracing hooks for Windows Performance Analyzer (WPA):

- **Event Markers**: Engine tick start/end, dirty rect invalidation count, telemetry cache updates.
- **Zero Overhead**: When ETW tracing is disabled by Windows kernel, provider calls decay into zero-overhead instructions.
