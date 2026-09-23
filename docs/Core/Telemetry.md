# Telemetry Subsystem (`system_providers`)

**Purpose**: Hardware sensor collectors, metrics data model, and lock-free shared cache.  
**Audience**: Engine Developers, Hardware Provider Contributors.  
**Prerequisites**: [DATA_FLOW.md](../Architecture/DATA_FLOW.md).  
**Related Documents**: [Scheduler.md](Scheduler.md).  
**Last Updated**: 2026-09-01  
**Status**: Active / Core Subsystem  
**Owner**: System Telemetry Team  

---

## 1. Hardware Sampler Inventory

| Collector | Metric Source | Implementation |
|---|---|---|
| `CpuProvider` | System Idle/Kernel/User Time | Win32 `GetSystemTimes` with Task Manager EMA smoothing (α=0.25), sub-quantum tick holding |
| `MemoryProvider` | Used/Total Physical RAM | Win32 `GlobalMemoryStatusEx` with EMA smoothing (α=0.3) |
| `GpuProvider` | VRAM Memory Budget Usage % | DXGI `IDXGIAdapter3::QueryVideoMemoryInfo` with EMA smoothing |
| `NetworkProvider` | Real-time Octet Throughput | Win32 `GetIfTable2` with time-scaled Δt and EMA smoothing |
| `BatteryProvider` | Charge %, Remaining Seconds, Charging State | Win32 `GetSystemPowerStatus` |
| `AudioProvider` | Master Volume %, Mute State | **Stub: returns hardcoded 75% (see BUG-001)** |
| `ProcessMetricsProvider` | Running Process Count & Category Estimation | Win32 `EnumProcesses` with heuristic app classification |
| `DisplayTopologyProvider` | Monitor Count, GPU Count, External/Virtual Displays | Win32 `GetSystemMetrics(SM_CMONITORS)` + DXGI adapter enumeration |
| `CpuTopologyProvider` | Logical/Physical Core Topology & P/E Cores | Win32 `GetLogicalProcessorInformationEx` |
| `DedicatedGpuProvider` | Dedicated/Shared VRAM & 3D Engine % | Windows D3DKMT & DXGI `IDXGIAdapter3::QueryVideoMemoryInfo` |
| `WasapiAudioProvider` | Real-Time 16-Band FFT & Peak dB | Windows Core Audio WASAPI Loopback & SMTC Session Manager |
| `CryptoFinancialProvider` | Live Financial & Crypto Asset Prices | Simulated price data (planned: real API integration) |
| `NetworkDiagnosticsProvider` | Ping Latency, Bandwidth Quality, Connection Status | Extended network diagnostics |

---

## 2. Core Architecture Principle — "Collect Once, Publish Everywhere"

A single `TelemetryService` pass samples all hardware sensors once per 10ms engine cycle and publishes an immutable `TelemetrySnapshot` into the lock-free `SharedTelemetryCache`. Widgets and background scripts read exclusively from `SharedTelemetryCache` without issuing repeated Windows API or driver queries.

---

## Known Issues
- None.

## References
- [crates/system_providers/src/shared_cache.rs](../../crates/system_providers/src/shared_cache.rs)
- [crates/system_providers/src/telemetry_service.rs](../../crates/system_providers/src/telemetry_service.rs)

## Related Documents
- [Engine.md](Engine.md)
