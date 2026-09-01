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
| `CpuProvider` | System Idle/Kernel/User Time | Win32 `GetSystemTimes` with Task Manager EMA smoothing |
| `CpuTopologyProvider` | Logical/Physical Core Topology & P/E Cores | Win32 `GetLogicalProcessorInformationEx` |
| `MemoryProvider` | Used/Total Physical RAM | Win32 `GlobalMemoryStatusEx` |
| `DedicatedGpuProvider` | Dedicated/Shared VRAM & 3D Engine % | Windows D3DKMT & DXGI `IDXGIAdapter3::QueryVideoMemoryInfo` |
| `NetworkProvider` | Real-time Octet Throughput | Win32 `GetIfTable2` |
| `WasapiAudioProvider` | Real-Time 16-Band FFT & Peak dB | Windows Core Audio WASAPI Loopback & SMTC Session Manager |

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
