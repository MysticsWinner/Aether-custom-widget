# Aether — Next-Generation Windows Desktop Customization Platform

[![Rust 2021](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Platform: Windows 11](https://img.shields.io/badge/Platform-Windows%2011%20(x86__64%2FARM64)-0078D4.svg)](https://www.microsoft.com/windows)
[![Tests: 333/333 Passing](https://img.shields.io/badge/Tests-333%2F333%20Passing-brightgreen.svg)](docs/Testing/TESTING.md)

An enterprise-class, hardware-accelerated, zero-trust desktop customization engine designed for ultra-low resource usage, instant responsiveness, and complete fault isolation on Windows 11.

---

## 🏛️ Master Documentation Portal

Welcome to the Aether Documentation System. The documentation is organized into clear domain trees designed for long-term maintainability, zero redundancy, and strict architectural governance.

```
docs/
├── Architecture/   ── Priority 2: System overview, GPU rendering, IPC protocol, data flow, threading, ADRs (001-012)
├── Development/    ── Priority 3: Coding standards, contributing guidelines, build guide, workspace layout, refactoring
├── Testing/        ── Priority 4: Testing governance, unit tests, integration tests, stress tests, QA checklist
├── Security/       ── Priority 5: Security architecture, STRIDE threat model, sandboxing, permissions, package signatures
├── Performance/    ── Priority 6: Master performance report, benchmark methodology, memory allocation analysis, profiling
├── API/            ── Priority 7: Rust Widget SDK, C# SDK, TypeScript SDK, Lua API, Plugin C ABI, IPC JSON API
├── Project/        ── Master project reports, living audit encyclopedia, roadmap, status, changelogs
├── Core/           ── Core subsystems (Engine, Scheduler, Telemetry, Plugins, AI, Cloud Sync, Theme, Layout)
├── Rendering/      ── Low-level rendering specs (Direct2D, DirectComposition, Dirty Regions, GPU Pipeline, WorkerW)
├── GUI/            ── WinUI 3 Dashboard (12 pages, MVVM), Ratatui TUI, Diagnostics & Settings
├── Platform/       ── OS Platform Support Matrix (Windows 11, Linux, macOS)
├── AI/             ── AI Engine vision, agent workflows, prompt library, limitations & verification
├── Release/        ── Production release notes, PR description templates, release process, versioning
└── archive/        ── Master release audit reports & historical verification logs
```

---

## 📍 Frequently Accessed Core Documents

| Document | Description | Path |
|:---|:---|:---|
| 📖 **Master Project Report** | Living encyclopedia & audit report ("Single Source of Truth") | [Detailed_Project_Report.md](docs/Project/Detailed_Project_Report.md) |
| 📐 **System Architecture** | Subsystem orchestrator, event bus, rendering and IPC topologies | [ARCHITECTURE.md](docs/Architecture/ARCHITECTURE.md) |
| 🏛️ **Architectural Decisions** | Formal Architectural Decision Records (ADR 001 through ADR 012) | [ADR.md](docs/Architecture/ADR.md) |
| 🔌 **Widget SDK Guide** | Standardized 6-pillar widget development API and lifecycle | [WIDGET_API.md](docs/API/WIDGET_API.md) |
| 🔐 **Security & Sandboxing** | Capability broker, AppContainer sandbox, and widget firewall | [SECURITY_ARCHITECTURE.md](docs/Security/SECURITY_ARCHITECTURE.md) |
| ⚡ **Performance & Benchmarks** | Empirical performance audit results vs legacy platforms | [PERFORMANCE_OVERVIEW.md](docs/Performance/PERFORMANCE_OVERVIEW.md) |
| 📊 **Feature Status & Matrix** | Subsystem completion status & benchmark metrics | [Project_Status.md](docs/Project/Project_Status.md) |
| 🧪 **Testing Protocol** | Mandatory testing protocol & 331-test automated harness | [TESTING.md](docs/Testing/TESTING.md) |
| 🛠️ **Build & Workspace Guide** | Build commands, toolchains, and 33 workspace crates inventory | [BUILD.md](docs/Development/BUILD.md) |

---

## ⚡ Performance Benchmarks vs Competitors

| Metric | Aether Platform | Rainmeter (Legacy) | Performance Gain | Reference |
|:---|:---|:---|:---|:---|
| **Idle CPU Usage (100 Widgets)** | **`< 0.08% CPU`** | 8.5% – 12.0% CPU | **40x Lower CPU** | [BENCHMARK_METHODOLOGY.md](docs/Performance/BENCHMARK_METHODOLOGY.md) |
| **Physical RAM Footprint** | **`< 22 MB RAM`** | 120 MB – 350 MB+ | **80%+ RAM Savings** | [PERFORMANCE_OVERVIEW.md](docs/Performance/PERFORMANCE_OVERVIEW.md) |
| **Cold Startup Latency** | **`< 45 ms`** | 1,650 ms | **37x Faster Boot** | [Startup_Shutdown.md](docs/Architecture/Startup_Shutdown.md) |
| **Max Refresh Rate** | **144 Hz+ Native** | 30 Hz – 60 Hz | **Zero Tear / 0.18ms Frame Time** | [CORE_RENDERING.md](docs/Architecture/CORE_RENDERING.md) |
| **Crash Fault Isolation** | **AppContainer Sandbox (< 5ms recovery)** | Full Process Crash | **100% Host Uptime** | [SANDBOX.md](docs/Security/SANDBOX.md) |

---

## 🛠️ Quick Start

```powershell
# Launch full stack (Daemon + TUI Dashboard):
.\launch.ps1

# Run core engine background daemon:
cargo run -p core_engine

# Run TUI dashboard:
cargo run -p dashboard_tui

# Run full workspace test suite (268 passing tests):
cargo test --workspace
dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj
```

---

## 📜 Governance

Please read our [Contributing Guidelines](docs/Development/CONTRIBUTING.md) and [AGENTS Governance Rules](.agents/AGENTS.md) before submitting pull requests.
