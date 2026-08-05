# Aether — Product Roadmap

**Document Status: Active Development Roadmap**

---

## Roadmap Overview

Aether is structured across a multi-phase development timeline transitioning the engine from a high-performance **Proof-of-Concept / Prototype** into a **Production-Ready Desktop Customization Platform**.

---

## Phase Milestones Overview

```mermaid
gantt
    title Aether Release Roadmap
    dateFormat  YYYY-MM-DD
    section Phase 1-5 Core
    Core Engine Daemon & Telemetry      :done, p1, 2026-01-01, 2026-03-01
    Widget SDK & IPC Protocol          :done, p2, 2026-03-01, 2026-04-15
    section Phase 6-10 GUI & Infrastructure
    WinUI 3 GUI & TUI Dashboards       :done, p3, 2026-04-15, 2026-06-01
    Subsystem Suite & Simulation Layer :done, p4, 2026-06-01, 2026-07-15
    section Phase 11-15 Hardening (Current)
    Production Release Candidate (RC)  :active, p5, 2026-07-15, 2026-09-01
    section Phase 16+ Hardware Native
    DirectComposition Desktop Hooking  :crit, p6, 2026-09-01, 2026-11-01
    AppContainer Sandboxing            :p7, 2026-11-01, 2027-01-01
```

---

## Near-Term Goals (Q3 2026 — Phase 15 RC Consolidation)

- [x] **Core Async Host**: 10 ms Tokio tick loop orchestrating 10 engine subsystems cleanly.
- [x] **Real-time Telemetry Pipe**: Win32 `GetSystemTimes` (CPU) and `GlobalMemoryStatusEx` (RAM) published to `SharedTelemetryCache`.
- [x] **Named Pipe IPC Server**: Multi-client JSON protocol supporting ping, metrics, widget loading, and subsystem diagnostics.
- [x] **WinUI 3 Dashboard Shell**: Complete C# application shell with Mica backdrops, live gauge visualizers, process management, and diagnostic logging.
- [x] **Ratatui Terminal Dashboard**: TUI client polling IPC pipe with animated gauges.
- [x] **Comprehensive Test Suite**: Maintain 100% test pass rate across all workspace crates (**87/87 tests passing**).

---

## Mid-Term Goals (Q4 2026 — Real Hardware & Compositing)

- [ ] **DirectComposition Desktop Surface Integration**: Connect native `workerw_hook.cpp` C++ layer to Rust `Direct2DRenderer` via FFI to render widgets behind Windows desktop icons.
- [ ] **Hardware DXGI/NVML GPU Telemetry**: Replace GPU sine wave provider with real DXGI performance counter telemetry.
- [ ] **Network Interface Telemetry**: Integrate `GetIfTable2` Win32 API into `NetworkProvider` for real-time bytes sent/received measurement.
- [ ] **AppContainer Sandbox Enforcement**: Transition `PluginSupervisor` from simulated PID state to Windows `CreateAppContainerProfile` and `AssignProcessToJobObject`.
- [ ] **Lua Runtime API Expansion**: Register SDK telemetry, canvas, and event functions within `mlua` environment.

---

## Long-Term Vision (2027+ — Ecosystem & Cloud)

- [ ] **CRDT Cloud Synchronization**: Wire `cloud_sync` manager to production REST/WebSocket backend services for seamless multi-device config sync.
- [ ] **Ed25519 Package Signature Verification**: Integrate `ring` cryptographic crate into `package_manager` for signed widget installation.
- [ ] **Local AI Model Runtime**: Replace keyword prompt matching in `ai_engine` with local ONNX runtime for intelligent layout synthesis.
- [ ] **TypeScript / Web Widget SDK**: Provide WebAssembly / Web-based widget rendering container.
