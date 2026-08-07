# Telemetry Subsystem (`system_providers`)

**Purpose**: Hardware sensor collectors, metrics data model, and lock-free shared cache.  
**Audience**: Engine Developers, Hardware Provider Contributors.  
**Prerequisites**: [Data_Flow.md](../01_Architecture/Data_Flow.md).  
**Related Documents**: [Scheduler.md](Scheduler.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: System Telemetry Team  

---

## 1. Hardware Sampler Inventory

| Collector | Metric Source | Implementation |
|---|---|---|
| `CpuProvider` | System Idle/Kernel/User Time | Win32 `GetSystemTimes` |
| `MemoryProvider` | Used/Total Physical RAM | Win32 `GlobalMemoryStatusEx` |
| `PowerAndAudioCollector` | Battery Charge & Volume % | Win32 `GetSystemPowerStatus`, WASAPI |
| `AppMetricsCollector` | Top-Level Windows & Apps | Win32 `EnumWindows` & Process Table |
| `GpuAndDisplayCollector` | GPU Devices & Displays | Win32 `EnumDisplayDevices`, DXGI |

---

## Future Work
- Add native AMD ADL & NVIDIA NVML GPU hardware counters.

## Known Issues
- None.

## References
- [crates/system_providers/src/shared_cache.rs](file:///d:/Code/Aether-custom-widget/crates/system_providers/src/shared_cache.rs)

## Related Documents
- [Engine.md](Engine.md)
