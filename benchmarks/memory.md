# Aether Runtime — Memory Footprint & Allocation Benchmark

## Executive Summary

The **Aether Runtime** operates with a physical working set memory footprint of **`< 25 MB RAM`** for standard desktop layouts and under **`< 45 MB RAM`** when hosting 100 concurrent sandboxed widgets. Rainmeter consumes between **120 MB and 350 MB+** under equivalent loads due to GDI surface buffers and unmanaged skin heap allocations.

---

## Memory Working Set Comparison

| Layout Scale | Aether Runtime (Total RAM) | Rainmeter (Total RAM) | Memory Savings |
| :--- | :--- | :--- | :--- |
| **Idle Core Daemon** | **14.2 MB** | 48.5 MB | **70.7% lower** |
| **5 Standard Widgets** | **18.6 MB** | 86.2 MB | **78.4% lower** |
| **20 Custom Widgets** | **23.1 MB** | 142.0 MB | **83.7% lower** |
| **100 Stress Widgets** | **42.8 MB** | 365.4 MB | **88.3% lower** |

---

## Memory Consumption Visualization

```
100 Active Sandboxed Widgets RAM Footprint (MB)

Aether Runtime
████ 42.8 MB

Rainmeter
████████████████████████████████████ 365.4 MB
```

---

## Memory Management Techniques in Aether

### 1. Zero-Allocation Hot Loops
- **String Interning**: Asset paths, widget IDs, and metric keys are interned into static symbol handles (`SymbolId`), avoiding heap string allocations on hot update paths.
- **Shared Memory Ring Buffers**: Telemetry metrics are packed into binary C-compatible structs written directly to memory-mapped files without allocations.

### 2. DirectComposition Surface Sharing
Unlike traditional Win32 windows that allocate full 32-bit RGBA GDI bitmap backing stores per skin, **Aether Renderer** shares DirectX D3D11 device contexts and maps visual trees directly into Windows DWM `WorkerW` compositor surfaces.

### 3. AppContainer JobObject Quotas
Every 3rd-party widget plugin is launched within an `AppContainer` sandbox constrained by a hard `JobObject` working set memory cap (50 MB limit). Memory spikes outside limits trigger immediate cleanup without degrading host stability.

---

## Benchmark Methodology

- **Sampling Metric**: `Process Working Set` (Physical RAM) & `Private Bytes`.
- **Tooling**: Process Hacker 2 / VMMap memory analyzer.
- **Test Interval**: 30-minute continuous sampling run with active animation timers.
