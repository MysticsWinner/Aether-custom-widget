# Benchmark Methodology & Standard Operating Procedures

This document outlines the rigorous testing methodology and hardware setup used to execute comparative performance benchmarks.

---

## 🖥️ Benchmark System Specifications

- **OS**: Windows 11 Pro 64-bit (Build 22631.3880)
- **CPU**: AMD Ryzen 9 7950X3D (16 Cores / 32 Threads @ 4.2GHz base)
- **RAM**: 64 GB DDR5-6000 MHz EXPO
- **GPU**: NVIDIA GeForce RTX 4090 (24GB VRAM, Driver 555.99)
- **Display**: 4K UHD (3840x2160) @ 144 Hz (HDR Enabled, G-Sync Active)

---

## 🔬 Benchmark Execution Procedure

1. **Clean Environment Isolation**: Terminate non-essential background processes and services. Disable Windows Defender real-time scanning hooks for benchmark directories.
2. **Warmup Phase**: Execute 1,000 tick passes to ensure CPU frequency scaling stabilizes and JIT/caching warms up.
3. **10,000 Sample Collection**: Record 10,000 continuous frame render cycles using ETW tracing and high-resolution performance counters (`QueryPerformanceCounter`).
4. **Rainmeter Comparative Test Suite**: Run Rainmeter 4.5.18 with default illustro skins vs CustomWidget with matching vector layouts.
