# Aether — Next-Generation Windows Desktop Customization Platform

[![Rust 2021](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Platform: Windows 11](https://img.shields.io/badge/Platform-Windows%2011%20(x86__64%2FARM64)-0078D4.svg)](https://www.microsoft.com/windows)
[![Tests: 184/184 Passing](https://img.shields.io/badge/Tests-184%2F184%20Passing-brightgreen.svg)](docs/08_Testing/Test_Structure.md)

An enterprise-class, hardware-accelerated, zero-trust desktop customization engine designed for ultra-low resource usage, instant responsiveness, and complete fault isolation.

---

## 🏛️ Master Documentation Portal

Welcome to the Aether Documentation System. The documentation is organized into 10 structured domains designed for long-term scalability and zero redundancy.

```
docs/
├── 00_Project/      ── Master Project Reports, Status & Architecture Overviews
├── 01_Architecture/ ── Core System Architecture, Threading, IPC & Memory Models
├── 02_Core/         ── Engine Subsystems (Scheduler, Telemetry, Plugins, AI, Sync)
├── 03_Rendering/    ── DirectComposition, Direct2D, WorkerW Hooks & GPU Pipeline
├── 04_SDK/          ── Multi-Language SDKs (Rust, Lua, C#, TypeScript, Plugin API)
├── 05_GUI/          ── WinUI 3 Dashboard, Ratatui TUI & Diagnostic Tools
├── 06_Platform/     ── OS Platform Support Matrix (Windows 11, Linux, macOS)
├── 07_Security/     ── Sandboxing, Permissions, Capability Broker & Threat Models
├── 08_Testing/      ── Automated Test Harness, Benchmarks & Stress Tests
└── 09_Development/  ── Build Instructions, Workspace Standards & Release Workflow
```

---

## 📍 Frequently Accessed Core Documents

| Document | Description | Path |
|---|---|---|
| 📖 **Master Project Report** | Living encyclopedia & audit report ("Single Source of Truth") | [Detailed_Project_Report.md](docs/00_Project/Detailed_Project_Report.md) |
| 📐 **System Architecture** | Subsystem orchestrator, event bus, and subsystem lifecycle | [System_Architecture.md](docs/01_Architecture/System_Architecture.md) |
| 🔌 **Widget SDK Guide** | Complete 6-pillar widget development API guide | [Widget_SDK.md](docs/04_SDK/Widget_SDK.md) |
| 🔐 **Security & Sandboxing** | Capability broker, AppContainer sandbox, and widget firewall | [Security_Model.md](docs/07_Security/Security_Model.md) |
| 📊 **Feature Status & Matrix** | Subsystem completion status & benchmark metrics | [Project_Status.md](docs/00_Project/Project_Status.md) |
| 🧪 **Testing Protocol** | Mandatory testing protocol & workspace test harness | [Test_Structure.md](docs/08_Testing/Test_Structure.md) |

---

## ⚡ Performance Benchmarks vs Competitors

| Metric | Aether Platform | Rainmeter (Legacy) | Performance Gain | Reference |
| :--- | :--- | :--- | :--- | :--- |
| **Idle CPU Usage (100 Widgets)** | **`< 0.1% CPU`** | 8.5% – 12.0% CPU | **40x Lower CPU** | [Benchmarks.md](docs/08_Testing/Benchmarks.md) |
| **Physical RAM Footprint** | **`< 25 MB RAM`** | 120 MB – 350 MB+ | **80%+ RAM Savings** | [Performance.md](docs/08_Testing/Performance.md) |
| **Cold Startup Latency** | **`< 45 ms`** | 1,650 ms | **37x Faster Boot** | [Startup_Shutdown.md](docs/01_Architecture/Startup_Shutdown.md) |
| **Max Refresh Rate** | **144 Hz+ Native** | 30 Hz – 60 Hz | **Zero Tear / 0.32ms Frame Time** | [Rendering.md](docs/01_Architecture/Rendering.md) |
| **Crash Fault Isolation** | **AppContainer Sandbox (< 5ms recovery)** | Full Process Crash | **100% Host Uptime** | [Sandboxing.md](docs/07_Security/Sandboxing.md) |

---

## 🛠️ Quick Start

```powershell
# Launch full stack (Daemon + TUI Dashboard):
.\launch.ps1

# Run core engine background daemon:
cargo run -p core_engine

# Run TUI dashboard:
cargo run -p dashboard_tui

# Run full workspace test suite (184/184 tests):
cargo test --workspace
```

---

## 📜 License & Governance

Aether is dual-licensed under [MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE).  
Please read our [Contributing Guidelines](docs/09_Development/Contributing.md) and [AGENTS Governance Rules](AGENTS.md) before submitting pull requests.
