# Next-Generation Windows Desktop Customization Platform (Rainmeter Successor)

A production-grade, enterprise-class, hardware-accelerated Windows desktop customization platform designed for ultra-low resource usage, absolute crash resiliency, and zero-trust plugin isolation.

## Key Architectural Highlights

- ⚡ **Autonomous Headless Core Daemon**: Written in Rust (`windows-rs`, `tokio`), running as a high-performance background precursor service completely decoupled from GUI apps.
- 🎨 **DirectComposition & Direct2D Hardware Acceleration**: Renders transparent, subpixel typography and vector widgets directly onto Windows DWM compositor surfaces (`WorkerW`) with <25 MB total memory footprint.
- 🔒 **Zero-Trust AppContainer Sandboxing**: 3rd-party plugins execute in hardware-isolated Windows `AppContainer` sandboxes with restricted tokens and strict `JobObject` CPU/RAM limits.
- 📜 **Lua 5.4 Embedded Scripting**: Rapid widget logic development via safe embedded Lua 5.4 runtime alongside native executable plugins.
- 📐 **Flexbox Layout Engine**: Responsive widget element placement driven by the `taffy` flexbox layout solver.
- 🖥️ **WinUI 3 Decoupled Dashboard**: Modern Windows 11 Fluent UI management dashboard connected via Win32 Named Pipes IPC.
- 🛠️ **Failure Injection & Redundancies**: Chaos engineering failure injectors with ETW Tracing and automatic self-healing recovery.
- 📦 **npm-like Package Manager CLI**: Native CLI (`install weather-widget`, `install spotify-widget`, `install taskbar-plus`) with Ed25519 cryptographic signature verification.
- ☁️ **End-to-End Encrypted Cloud Sync**: CRDT conflict resolution with Lamport Vector Clocks and Offline-First Local Cache.
- 🤖 **AI Subsystem**: Voice processing (`VoiceIntentParser`), desktop automation, layout/theme/widget synthesis, and workflow rule automation.

## Repository Workspace Structure

```
Cutom-widget/
├── Cargo.toml                    # Master Workspace Configuration
├── README.md                     # Platform Overview
├── docs/                         # Comprehensive Engineering Documentation
│   ├── ARCHITECTURE.md           # Master Architecture Specification
│   ├── ADR.md                    # Architecture Decision Records (ADRs)
│   ├── THREAT_MODEL.md           # Threat Model & STRIDE Analysis
│   ├── PERFORMANCE_REPORT.md     # Master Performance Audit Report
│   ├── BENCHMARK_METHODOLOGY.md  # Standard Operating Procedures & Setup
│   ├── PROFILING_RESULTS.md      # CPU Sampling & Flamegraph Analysis
│   ├── MEMORY_ALLOCATION_ANALYSIS.md# Zero-Alloc Hot Loops & RAM Analysis
│   ├── IPC_DESIGN.md             # Named Pipes & Shared Memory Ring Buffers
│   ├── RENDERING_PIPELINE.md     # DirectComposition & Dirty Rect Culling
│   ├── WIDGET_SDK_GUIDE.md       # Multi-Language Widget SDK Manual
│   ├── THEMING_SPECIFICATION.md  # theme.json Schema & Token Reference
│   ├── SECURITY_AND_SANDBOXING.md# AppContainer & Security Spec
│   ├── MARKETPLACE_CLI.md        # Package Manager & CLI Reference
│   ├── PERFORMANCE_AND_BENCHMARKS.md# Profiler & Rainmeter Comparisons
│   ├── CLOUD_SYNC_SPEC.md        # Encrypted Cloud Sync & CRDT Spec
│   └── AI_SUBSYSTEM.md           # AI Subsystem & Workflow Spec
├── crates/
│   ├── core_engine/              # Primary Headless Daemon (Rust)
│   ├── plugin_runtime/           # AppContainer Sandbox & Process Manager
│   ├── lua_runtime/              # Embedded Lua 5.4 Host Bindings
│   ├── ipc_protocol/             # Shared Memory & Named Pipe Ring Buffers
│   ├── layout_engine/            # Taffy Layout Integrator
│   ├── theme_engine/             # Color Palette & Token Resolver
│   ├── animation_engine/         # Spring Physics & Easing Curves
│   ├── system_providers/         # PDH / NVML Hardware Metric Collectors
│   ├── widget_parser/            # TOML Schema & Expression Evaluator
│   ├── widget_sdk/               # Multi-Language Master Widget SDK
│   ├── package_manager/          # npm-like Package Manager CLI & Security
│   ├── cloud_sync/               # CRDT Encrypted Cloud Sync & Offline Queue
│   ├── ai_engine/                # Voice, Generation & Workflow Engine
│   └── production_engine/       # Security Audit, Stress Testing & Release Suite
├── bindings/
│   ├── csharp/CustomWidget.SDK/  # C# .NET 8 / WinUI 3 SDK Assembly
│   └── typescript/custom-widget-sdk/ # TypeScript @types Definitions Package
├── tests/                        # Master Integration Test Suite
├── src_gui/                      # WinUI 3 Management Dashboard (C# / WinUI 3)
└── native/win32_hooks/           # Native C++ DLL for Win32 Shell Hooks
```

## 📚 Platform Engineering Documentation Index

- **[Architecture Decision Records (ADRs)](docs/ADR.md)**: Architectural decisions ADR 001 through ADR 005.
- **[Threat Model & STRIDE Analysis](docs/THREAT_MODEL.md)**: STRIDE analysis and AppContainer security controls.
- **[Performance Audit Report](docs/PERFORMANCE_REPORT.md)**: Master NFR metrics compliance audit results.
- **[Benchmark Methodology](docs/BENCHMARK_METHODOLOGY.md)**: Hardware setup and standard operating procedures.
- **[Profiling Results & Flamegraphs](docs/PROFILING_RESULTS.md)**: CPU sampling profiles and flamegraph analyses.
- **[Memory Allocation Analysis](docs/MEMORY_ALLOCATION_ANALYSIS.md)**: RAM working set analysis and zero-alloc hot loops.
- **[IPC Design Specification](docs/IPC_DESIGN.md)**: Dual-channel Win32 Named Pipes and Shared Memory ring buffers.
- **[Rendering Pipeline](docs/RENDERING_PIPELINE.md)**: DirectComposition visual trees and Dirty Rectangle culling.
- **[Master Architecture Specification](docs/ARCHITECTURE.md)**: Deep dive into system layers and Tokio event daemon topology.
- **[Widget SDK Developer Guide](docs/WIDGET_SDK_GUIDE.md)**: Multi-language SDK manual (Rust, C#, TypeScript).
- **[Theme Engine Specification](docs/THEMING_SPECIFICATION.md)**: `theme.json` design tokens and zero-restart hot reloading.
- **[Security & Sandboxing Specification](docs/SECURITY_AND_SANDBOXING.md)**: AppContainer isolation, JobObjects, and Ed25519 signatures.
- **[Marketplace CLI Reference](docs/MARKETPLACE_CLI.md)**: Package manager CLI (`install weather-widget`, `install spotify-widget`, `install taskbar-plus`).
- **[Performance & Benchmark Matrix](docs/PERFORMANCE_AND_BENCHMARKS.md)**: 13-Metric profiler and Rainmeter comparisons.
- **[Encrypted Cloud Sync Specification](docs/CLOUD_SYNC_SPEC.md)**: CRDT Vector Clock conflict resolution and offline queueing.
- **[AI Subsystem & Intelligence Guide](docs/AI_SUBSYSTEM.md)**: Voice intent processing, synthesis, and workflow automation.
