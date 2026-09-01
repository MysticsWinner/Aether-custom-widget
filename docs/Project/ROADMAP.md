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
    section Phase 11-16 RC & Diagnostics
    Diagnostics & System Integration   :done, p5, 2026-07-15, 2026-08-15
    section Phase 17-26 v0.7 Roadmap
    Phase 17-18: Crash Recovery & Config Safety :active, p6, 2026-09-01, 2026-11-01
    Phase 19-21: Security Depth & Reliability   :p7, 2026-11-01, 2027-02-01
    Phase 22-26: Perf, DevTools & Wow Features :p8, 2027-02-01, 2027-08-01
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

- [x] **Aether 7.4 Master Release (Design System + Adaptive Visual Platform)**:
  - [x] **Semantic Design Token System**: 12-category semantic token hierarchy (Colors, Typography, Spacing, Sizing, Shape, Borders, Elevation, Materials, Motion, Opacity, Accessibility, Performance).
  - [x] **Cascading Theme Engine**: Theme inheritance (`extends`) with cycle detection and microsecond lookup.
  - [x] **Material Engine & Glass/Acrylic/Mica**: Material abstraction pipeline with hardware/battery/accessibility performance degradation.
  - [x] **Dynamic Wallpaper & WCAG Contrast**: Wallpaper palette extraction and WCAG 2.1 AA / APCA contrast validation.
  - [x] **Reactive Widget Architecture**: Signal-driven event bindings (`Telemetry/Event -> Signal -> Binding -> Dirty Region -> Render`).
  - [x] **Desktop Profiles & Context-Aware Engine**: Atomic profile switching (`Gaming`, `Coding`, `Streaming`, `Work`, `Minimal`, `Travel`, `Custom`) with context signal auto-triggers.
  - [x] **Declarative Widgets & Inspector**: Manifest-driven static widgets and extended 7.4 visual inspector.
  - [x] **AI Desktop Composer**: Natural language intent composition with mandatory capability validation gate.
  - [x] **100% Test Coverage & ADRs**: ADRs 006–012 created, 144/144 workspace unit and integration tests passing.

