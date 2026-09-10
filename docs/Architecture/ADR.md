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
| [ADR 013](#adr-013-wait-free-atomic-telemetry-publishing--double-buffered-seqlock-snapshots) | Wait-Free Atomic Telemetry Publishing & Double-Buffered Seqlock Snapshots | **Accepted** | Telemetry Synchronization |
| [ADR 014](#adr-014-subsystem-cadence-scheduling--failure-aware-cleanup-rollback) | Subsystem Cadence Scheduling & Failure-Aware Cleanup Rollback | **Accepted** | Host Engine Lifecycle |
| [ADR 015](#adr-015-authoritative-directcomposition-pipeline--workerw-shell-recovery) | Authoritative DirectComposition Pipeline & WorkerW Shell Recovery | **Accepted** | Authoritative Rendering & Recovery |
| [ADR 016](#adr-016-multi-reader-spmc-shared-memory-ipc-protocol) | Multi-Reader SPMC Shared-Memory IPC Protocol | **Accepted** | IPC & Memory-Mapped Protocol |
| [ADR 017](#adr-017-classified-event-reliability-semantics--state-reconstruction) | Classified Event Reliability Semantics & State Reconstruction | **Accepted** | Event Bus & State Recovery |

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

---

## ADR 013: Wait-Free Scalar Reads, Lock-Free Seqlock Snapshots & Cache Coherence Reality

* **Status**: Accepted
* **Context**: Telemetry previously claimed "lock-free `RwLock`" access, but `std::sync::RwLock` is a blocking kernel/futex primitive. Furthermore, documentation loosely used "wait-free" to describe composite snapshot reads which can retry under concurrent writes.
* **Decision**: 
  - Implement **wait-free** atomic scalar registers (`AtomicU32` storing IEEE-754 bits, `AtomicU64`, `AtomicBool`) for single-metric fast paths (`get_cpu_pct()`, `get_memory_used_mb()`), guaranteeing bounded $O(1)$ instructions with zero retries.
  - Implement **lock-free** seqlock-style double-buffered snapshot acquisition (`get_snapshot()`), eliminating reader-reader locking and mutexes with bounded spin loop retries.
  - Explicitly document the physical hardware contention model: while software mutex contention is eliminated, cache-coherence bus traffic (MESI/MOESI Read-For-Ownership and store buffer drain cycles) still occurs across CPU cores on writer updates.
* **Consequences**:
  - *Positive*: Fast-path metric queries are mathematically wait-free with zero lock contention.
  - *Positive*: Full telemetry snapshots are consistent and tear-free without blocking.
  - *Positive*: Concurrency-theory documentation is rigorous and technically auditable.

---

## ADR 014: Subsystem Cadence Scheduling, Deadline Tracking & Lifecycle Separation

* **Status**: Accepted
* **Context**: All subsystems previously executed on a monolithic synchronous tick loop, conflating core daemon lifecycle with widget plugin execution. Furthermore, partial startup failures leaked system resources.
* **Decision**: 
  - Formally separate the **9-state Subsystem Lifecycle Machine** (`Uninitialized`, `Starting`, `Ready`, `Degraded`, `Recovering`, `Stopping`, `Stopped`, `Failed`, `SafeMode`) from the **11-state Widget Plugin Lifecycle Machine** (`Unloaded`, `Loading`, `Loaded`, `Mounting`, `Active`, `Paused`, `Degraded`, `Unmounting`, `Unloading`, `Error`, `Quarantined`).
  - Implement `SubsystemManager` with categorized cadences (`Periodic`, `Reactive`, `OnDemand`, `DeadlineDriven`) and real-time execution statistics (`SubsystemExecutionStats`) tracking tick durations and deadline misses.
  - Implement automatic reverse-order cleanup rollback (`shutdown()`) if any subsystem fails during `initialize_all()`.
* **Consequences**:
  - *Positive*: Eliminates lifecycle terminology ambiguity.
  - *Positive*: Prevents resource leaks on startup failures.
  - *Positive*: Provides runtime deadline miss tracking and scheduler fairness between subsystems.

---

## ADR 015: Authoritative DirectComposition Pipeline & WorkerW Shell Recovery

* **Status**: Accepted
* **Context**: Rendering architecture must establish an authoritative primary pipeline versus compatibility fallbacks, and survive Windows Explorer shell crashes without losing desktop icon backing.
* **Decision**: Formalize DirectComposition visual tree hosting on Direct3D 11 / Direct2D device contexts as the authoritative primary pipeline; isolate `UpdateLayeredWindow` layered HWND as a secondary compatibility fallback. Implement `DesktopSurfaceManager` with active `WorkerWSurfaceState` tracking, detecting Explorer restart events and automatically re-querying `Progman` (message `0x052C`) to rebind the WorkerW surface.
* **Consequences**:
  - *Positive*: Seamless desktop widget survival across Explorer.exe crashes and shell restarts.
  - *Positive*: DirectComposition GPU acceleration maintains sub-pixel typography and Mica/Acrylic effects.

---

## ADR 016: Multi-Reader SPMC Shared-Memory IPC Protocol & Win32 SDDL Security

* **Status**: Accepted
* **Context**: Shared-memory IPC requires safe concurrency for multiple readers (WinUI 3 GUI, ratatui TUI, sandboxed widgets) without pointer corruption or unprivileged tampering from untrusted plugins.
* **Decision**: 
  - Implement `MultiReaderSnapshotBuffer` and `MultiReaderRingBuffer` with a monotonic producer sequence counter and independent per-reader local cursors (`MultiReaderCursor`).
  - Apply the Win32 SDDL string `D:(A;;FR;;;AC)(A;;FR;;;WD)(A;;FA;;;BA)(A;;FA;;;SY)S:(ML;;NW;;;LW)` granting AppContainer sandboxed processes read-only access at Low Integrity level.
  - Implement `WriterCrashedMidWrite` detection to prevent infinite spins if a producer terminates mid-write, and fixed 64-byte cross-architecture layouts to eliminate 32/64-bit alignment mismatches.
* **Consequences**:
  - *Positive*: Arbitrary concurrent reader processes read telemetry simultaneously with zero reader-reader contention and zero pointer corruption.
  - *Positive*: Untrusted AppContainer plugins cannot write to or corrupt shared memory.

---

## ADR 017: Three-Tier Event Taxonomy (Ephemeral, Replayable, Durable) & Overflow Gap Reconciliation

* **Status**: Accepted
* **Context**: Using an unclassified broadcast channel causes missed critical state transitions or unbounded memory consumption. Furthermore, calling in-memory buffers "durable" conflates volatility with persistence, and late-joining subscribers risked silent desynchronization on buffer overflows.
* **Decision**: 
  - Classify events into three distinct tiers: `Ephemeral` (10ms transient ticks), `Replayable` (in-memory 128-entry sequence buffer), and `Durable` (WAL/disk-persisted configuration).
  - Implement overflow gap detection: when a subscriber requests `replay_since(seq)` where $seq + 1 < \text{oldest\_available}$, the bus returns `ReplayResult::GapDetected` containing an authoritative `AuthoritativeStateSnapshot` for state reconciliation instead of silent partial history.
* **Consequences**:
  - *Positive*: Reconnecting GUI clients detect evicted events and perform clean snapshot state reconciliation.
  - *Positive*: Accurate concurrency and persistence terminology.

---

## ADR 018: AI Mutation Security Pipeline, Capability Gate & Evasion Hardening

* **Status**: Accepted
* **Context**: Untrusted AI proposals (from voice recognition, natural language layout composers, workflow automation, or plugin scripts) can introduce prompt injections, path traversal evasions, or unauthorized system configuration mutations.
* **Decision**:
  - Implement `AiSecurityGate` enforcing a mandatory 4-stage pipeline: `SchemaValidator` -> `PolicyValidator` -> `CapabilityValidator` -> `HumanApprovalGate`.
  - Harden path validation against evasion: normalize URL-encoded tokens (`%2e%2e`, `%2f`, `%5c`), block embedded null bytes (`\0`), block UNC network shares (`\\server\share`), reject absolute drive roots, and enforce directory whitelisting and `.toml` extensions.
  - Require explicit human confirmation (`user_confirmed: true`) for all mutating operations (`LoadWidget`, `ApplyConfig`), blocking alternate execution paths from bypassing the security gate.
* **Consequences**:
  - *Positive*: Prompt injection, encoded directory traversals, and unapproved configuration mutations are strictly blocked.
  - *Positive*: Human-in-the-loop governance guarantees user sovereignty over desktop mutations.

