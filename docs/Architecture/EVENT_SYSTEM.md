# Event System Architecture

**Purpose**: Technical documentation of Aether's central broadcast event bus (`CoreEvent`) and reliable event delivery architecture.  
**Audience**: Engine Developers, Plugin Developers, Systems Architects.  
**Prerequisites**: [ARCHITECTURE.md](ARCHITECTURE.md), [Threading_Model.md](Threading_Model.md).  
**Related Documents**: [Data_Flow.md](Data_Flow.md), [IPC_PROTOCOL.md](IPC_PROTOCOL.md).  
**Last Updated**: 2026-09-07  
**Status**: Active / Production Technical Specification  
**Owner**: Core Engine Team  

---

## 1. Classified Event Taxonomy (`CoreEvent`)

Subsystems and widgets communicate asynchronously via `EventBus` (`tokio::sync::broadcast`). To maintain rigorous concurrency semantics, Aether distinguishes three distinct tiers of event lifecycle:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventReliability {
    /// High-frequency transient ticks (e.g. TelemetryTick).
    /// If a subscriber falls behind, ephemeral events are safely dropped to preserve latency.
    Ephemeral,
    /// In-memory sequenced state transitions (Theme, Profile, Widget Lifecycle, System State).
    /// Assigned a monotonic sequence number and retained in the 128-entry replay buffer.
    /// Recoverable for recent reconnections, but not persisted to disk.
    Replayable,
    /// Authoritative state changes persisted to disk via WAL / ConfigTransaction.
    /// Survives daemon restarts (e.g. license grants, user configuration commits).
    Durable,
}
```

### Event Variants & Classification:
| Event Variant | Reliability Class | Persistence / Delivery | Description |
|:---|:---|:---|:---|
| `TelemetryTick(TelemetrySnapshot)` | `Ephemeral` | Dropped on lag | 10 ms hardware metric snapshot |
| `ThemeChanged(String)` | `Replayable` | 128-slot Replay buffer & State cache | Global theme / design token change |
| `WidgetLoaded { widget_id }` | `Replayable` | 128-slot Replay buffer | Widget mounted to desktop canvas |
| `WidgetUnloaded { widget_id }` | `Replayable` | 128-slot Replay buffer | Widget detached from canvas |
| `SystemStateChanged(SystemState)` | `Replayable` | 128-slot Replay buffer & State cache | Power, fullscreen, or safe-mode transitions |
| `ProfileSwitched { profile_name }`| `Replayable` | 128-slot Replay buffer & State cache | Context-aware profile activations |
| `CapabilityRevoked { token_id }`  | `Replayable` | 128-slot Replay buffer | Security capability token revocation |
| `SubsystemSignal { signal }`      | `Replayable` | 128-slot Replay buffer | State degradation / health notifications (`STATE_DEGRADED`, etc.) |
| `ConfigCommitted { key, value }`   | `Durable`    | Atomic Disk Transaction (`ConfigManager`) | Persisted desktop layout & user settings |

### Structured Logging & Observability:
- Every `publish()` call logs subscriber count, event reliability tier, and delivery state.
- Unsubscribed/idle drops for ephemeral ticks are logged cleanly at `trace` level.
- Replay operations log requested sequences, continuous catch-up lengths, and structured warnings on gap detection.

---

## 2. Event Replay, Overflow Semantics & State Reconstruction

```mermaid
graph TD
    Publisher[Subsystem / Engine Component] -->|publish| EventBus
    
    subgraph EventBus ["EventBus Subsystem"]
        Classify{Taxonomy Filter}
        BroadcastChan["tokio::sync::broadcast (capacity: 256)"]
        ReplayBuf["Monotonic Replay Buffer (capacity: 128)"]
        StateCache["Authoritative State Cache"]
    end
    
    EventBus --> Classify
    Classify -->|All Events| BroadcastChan
    Classify -->|Replayable & Durable| ReplayBuf
    Classify -->|State Mutators| StateCache
    
    BroadcastChan --> ActiveSubscribers[Live Subscribed Tasks]
    
    Subscriber[Reconnecting GUI / Subscriber] -->|replay_since(seq)| ReplayBuf
    ReplayBuf -->|Continuous| DeliverEvents[Deliver Missed Events]
    ReplayBuf -->|Gap Detected (seq < oldest)| ForceSnapshot[Return ReplayResult::GapDetected + AuthoritativeStateSnapshot]
    ForceSnapshot --> ReconcileState[Subscriber Reconciles Authoritative State]
```

### 2.1 Replay Buffer Overflow & Gap Detection (`ReplayResult`)
A 128-event replay buffer provides rapid catch-up for brief network or thread hiccups, but if a subscriber disconnects for an extended period (e.g. WinUI 3 dashboard minimized or restarted):
$$\text{requested\_seq} + 1 < \text{oldest\_available\_seq}$$
Rather than silently replaying an incomplete, corrupted sequence of events, the `EventBus` explicitly detects the gap and returns a structured `ReplayResult`:

```rust
#[derive(Debug, Clone)]
pub enum ReplayResult {
    /// The requested sequence was within the retained window; events are continuous.
    Continuous(Vec<SequencedEvent>),
    /// A buffer overflow occurred; requested sequence was evicted.
    /// Forces subscriber to adopt the authoritative state snapshot rather than guessing.
    GapDetected {
        requested_seq: u64,
        oldest_available_seq: u64,
        snapshot: AuthoritativeStateSnapshot,
    },
}
```

When `ReplayResult::GapDetected` is returned, the caller discards its stale local state and reconciles directly from the bundled `AuthoritativeStateSnapshot`:
- `system_state: SystemState`
- `active_theme: Option<String>`
- `active_profile: Option<String>`
- `reconciled_seq: u64`

### 2.2 Authoritative State Cache
The `EventBus` continuously synchronizes its internal state cache on every incoming mutator event:
- `current_system_state() -> SystemState`
- `current_theme() -> Option<String>`
- `current_profile() -> Option<String>`
- `authoritative_snapshot() -> AuthoritativeStateSnapshot`

Late-joining subscribers can immediately acquire this snapshot with zero latency without waiting for future broadcast events.

---

## 3. Concurrency & Performance Guarantees

- **Non-blocking Dispatch**: `publish()` is synchronous and completes in `< 3 µs`.
- **Zero Reader Contention**: Subscribers consume from independent `broadcast::Receiver` queues.
- **Memory Bounded**: Replay buffer is strictly bounded to 128 entries; ephemeral events bypass the replay buffer completely.

---

## References
- [crates/core_engine/src/event_bus.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/event_bus.rs)
- [crates/core_engine/src/subsystems.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/subsystems.rs)
