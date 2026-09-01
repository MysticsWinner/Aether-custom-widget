# Architecture Decision Records (ADRs)

This document captures the formal architectural decisions made during the design and engineering of the Next-Generation Windows Desktop Customization Platform (**Aether**).

---

## Index of Architectural Decision Records

| ADR | Title | Status | Impact Area |
|:---|:---|:---|:---|
| [ADR 001](#adr-001-autonomous-headless-rust-background-daemon) | Autonomous Headless Rust Background Daemon | **Accepted** | Core Host Lifecycle |
| [ADR 002](#adr-002-directcomposition--direct2d-graphics-pipeline) | DirectComposition & Direct2D Graphics Pipeline | **Accepted** | GPU Compositing & Direct2D |
| [ADR 003](#adr-003-out-of-process-appcontainer-plugin-sandboxing) | Out-of-Process AppContainer Plugin Sandboxing | **Accepted** | Security & Sandboxing |
| [ADR 004](#adr-004-collect-once-publish-everywhere-telemetry-service) | "Collect Once, Publish Everywhere" Telemetry Service | **Accepted** | Hardware Telemetry & Caching |
| [ADR 005](#adr-005-state-based-crdts-for-encrypted-cloud-synchronization) | State-Based CRDTs for Encrypted Cloud Synchronization | **Accepted** | Cloud Sync & State Merging |
| [ADR 006](#adr-006-aether-design-token-system) | Aether Design Token System | **Accepted** | Theme Engine & Design Tokens |
| [ADR 007](#adr-007-theme-inheritance--cascading-resolution-architecture) | Theme Inheritance & Cascading Resolution Architecture | **Accepted** | Theme Engine & Inheritance |
| [ADR 008](#adr-008-material-engine--adaptive-fallback-pipeline) | Material Engine & Adaptive Fallback Pipeline | **Accepted** | Visual Materials & Compositor |
| [ADR 009](#adr-009-reactive-signal-binding-widget-architecture) | Reactive Signal-Binding Widget Architecture | **Accepted** | Widget SDK & Reactive Bindings |
| [ADR 010](#adr-010-adaptive-performance-budgets--degradation-hierarchy) | Adaptive Performance Budgets & Degradation Hierarchy | **Accepted** | Performance & Resource Quotas |
| [ADR 011](#adr-011-desktop-profiles--context-aware-automation-engine) | Desktop Profiles & Context-Aware Automation Engine | **Accepted** | Config Manager & Context Detection |
| [ADR 012](#adr-012-ai-desktop-composer--mandatory-security-validation-gate) | AI Desktop Composer & Mandatory Security Validation Gate | **Accepted** | AI Engine & Security Validation |

---

## ADR 001: Autonomous Headless Rust Background Daemon

* **Status**: Accepted
* **Context**: Legacy desktop widget platforms (e.g., Rainmeter) run as a single monolithic GUI process. If the UI thread freezes or crashes, widget rendering and background timers fail.
* **Decision**: Implement a headless precursor service daemon in Rust (`windows-rs`, `tokio`). The daemon runs autonomously in the background independent of any management GUI windows.
* **Consequences**:
  - *Positive*: Complete stability; closing GUI windows never terminates desktop widgets.
  - *Positive*: Ultra-fast startup (<45 ms cold boot).
  - *Negative*: Requires explicit IPC protocol (`\\.\pipe\CustomWidgetEngineControlPipe`) between host daemon and management GUI.

---

## ADR 002: DirectComposition & Direct2D Graphics Pipeline

* **Status**: Accepted
* **Context**: GDI+ software rendering causes high CPU consumption, screen tearing, and frame pacing issues during animations.
* **Decision**: Render transparent vector graphics and typography using Microsoft **DirectComposition** + **Direct2D 1.1** targeting `WorkerW` Desktop Window Manager (DWM) compositor surfaces.
* **Consequences**:
  - *Positive*: 60/120/144 Hz hardware acceleration with subpixel text rendering.
  - *Positive*: 92.4% redraw culling efficiency via dirty rectangle tracking (`DirtyRegionTracker`).
  - *Negative*: Windows 10/11 DWM specific API dependency.

---

## ADR 003: Out-of-Process AppContainer Plugin Sandboxing

* **Status**: Accepted
* **Context**: 3rd-party widget skins executing in-process can crash the host application, leak memory, or steal sensitive user data.
* **Decision**: Execute 3rd-party plugins out-of-process in restricted Windows **AppContainer** sandboxes under low-integrity SIDs with `JobObject` CPU (2%) and RAM (50 MB) limits.
* **Consequences**:
  - *Positive*: Complete crash fault isolation. A plugin crash never crashes the host daemon.
  - *Positive*: Strict security permission model (`widget.toml` capabilities enforced by `capability_broker`).
  - *Negative*: Slight IPC serialization overhead (<10 µs per message).

---

## ADR 004: "Collect Once, Publish Everywhere" Telemetry Service

* **Status**: Accepted
* **Context**: Multiple widgets querying OS APIs (PDH, NVML, `GetSystemTimes`, `GlobalMemoryStatusEx`) independently cause excessive kernel context switches and CPU spikes.
* **Decision**: Single background telemetry worker thread samples hardware metrics once per sampling interval (10 ms cycle) and commits to `SharedTelemetryCache` memory. Widgets read from the shared cache with zero repeated OS API calls.
* **Consequences**:
  - *Positive*: Idle CPU overhead dropped from 2.5% to <0.08%.
  - *Positive*: Eliminates redundant kernel sys-calls and context switches (reduced to <12/sec).

---

## ADR 005: State-Based CRDTs for Encrypted Cloud Synchronization

* **Status**: Accepted
* **Context**: Synchronizing layout bounds, active profiles, and themes across multiple monitors and workstations causes concurrent edit conflicts.
* **Decision**: Implement state-based Conflict-Free Replicated Data Types (CRDTs) with Lamport Vector Clocks and client-side AES-256-GCM encryption.
* **Consequences**:
  - *Positive*: Conflict-free deterministic merges across multiple devices without centralized locking.
  - *Positive*: Offline-first local SQLite WAL cache buffering.
  - *Negative*: Requires maintaining vector clocks per sync item.

---

## ADR 006: Aether Design Token System

* **Status**: Accepted
* **Context**: Legacy widget systems rely on hardcoded colors and static fonts, leading to fragmented visual themes and breaking consistency across widget skins.
* **Decision**: Implement a first-class, 12-category semantic Design Token architecture in `theme_engine`. Tokens encapsulate Colors, Typography, Spacing, Sizing, Shape, Borders, Elevation, Materials, Motion, Opacity, Accessibility, and Performance parameters.
* **Consequences**:
  - *Positive*: Complete visual consistency across all widgets; central theme changes instantly propagate to widgets via variable substitution (e.g., `{colors.accent}`).
  - *Positive*: Zero visual regressions during theme hot reloading.
  - *Negative*: Widgets must consume semantic tokens rather than raw hex values.

---

## ADR 007: Theme Inheritance & Cascading Resolution Architecture

* **Status**: Accepted
* **Context**: Creating modular themes requires duplicating entire token dictionaries, leading to high maintenance overhead.
* **Decision**: Implement cascading theme resolution: `System Defaults -> Base Theme -> Derived Theme -> Widget Theme -> Component Theme -> Instance Override`, with cycle detection (`detect_cycle`).
* **Consequences**:
  - *Positive*: Child themes only specify overridden tokens, dramatically reducing redundancy.
  - *Positive*: Prevents infinite recursion loops during theme inheritance resolution.
  - *Negative*: Cascading token lookup requires deterministic precedence evaluation.

---

## ADR 008: Material Engine & Adaptive Fallback Pipeline

* **Status**: Accepted
* **Context**: Complex blur, Acrylic, and Mica effects can cause frame drops on lower-end GPUs, laptops running on battery power, or remote desktop sessions.
* **Decision**: Create an explicit `MaterialEngine` abstraction providing material types (`Solid`, `Transparent`, `Glass`, `Acrylic`, `Mica`, `Elevated`, `Custom`) and an adaptive degradation pipeline (`Advanced Material -> GPU Check -> Power State Check -> Accessibility Check -> Fallback Material`).
* **Consequences**:
  - *Positive*: Preserves Aether's strict performance guarantees (<0.08% idle CPU) while providing visual effects.
  - *Positive*: Seamlessly respects High Contrast and Reduce Transparency Windows accessibility settings.

---

## ADR 009: Reactive Signal-Binding Widget Architecture

* **Status**: Accepted
* **Context**: Continuously polling telemetry metrics in tick loops causes unnecessary redraws and CPU context switches for static components.
* **Decision**: Implement `Signal<T>` and `SignalBinding` in `widget_sdk`, driving widget updates via metric signals (`Telemetry/Event -> Signal -> Binding -> Widget State -> Dirty Region -> Render`).
* **Consequences**:
  - *Positive*: Redraws occur exclusively when bound metric values change beyond hysteresis thresholds.
  - *Positive*: Zero allocation in signal version comparisons.

---

## ADR 010: Adaptive Performance Budgets & Degradation Hierarchy

* **Status**: Accepted
* **Context**: Misbehaving or unoptimized 3rd-party widget code can consume excessive CPU or memory without warning.
* **Decision**: Introduce `PerformanceBudget` declarations and `BudgetEvaluator` tracking declared vs actual resource consumption. State machine (`Normal -> SoftLimit -> Warning -> Degraded -> HardLimit`) automatically degrades visual effects or throttles rendering before host performance is impacted.
* **Consequences**:
  - *Positive*: Host system protection against resource starvation and memory leaks.
  - *Positive*: Proactive feedback to widget developers via dev tools inspector.

---

## ADR 011: Desktop Profiles & Context-Aware Automation Engine

* **Status**: Accepted
* **Context**: User desktop requirements change depending on activities (e.g. gaming vs coding vs streaming vs travelling).
* **Decision**: Implement `ProfileManager` (`Gaming`, `Coding`, `Streaming`, `Work`, `Minimal`, `Travel`, `Custom`) and `ContextAwareEngine` in `config_manager` to automatically detect context signals (fullscreen apps, battery saver, active processes) and trigger atomic profile switches.
* **Consequences**:
  - *Positive*: Seamless adaptation of desktop layout, active widgets, materials, and refresh rates based on active context.
  - *Positive*: Full rollback recovery on profile switch failure.

---

## ADR 012: AI Desktop Composer & Mandatory Security Validation Gate

* **Status**: Accepted
* **Context**: Synthesizing complete desktop themes and layouts via AI natural language prompts could bypass schema validation or security boundaries.
* **Decision**: Implement `AiDesktopComposer` in `ai_engine`, routing prompt synthesis through strict schema validation, capability checks (`capability_broker`), and performance prediction before presenting output for explicit user approval.
* **Consequences**:
  - *Positive*: AI-generated desktop layouts cannot execute unauthorized commands or violate platform performance/security boundaries.
  - *Positive*: User retains full approval control before changes are applied.
