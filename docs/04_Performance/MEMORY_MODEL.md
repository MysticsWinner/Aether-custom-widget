# Aether — Memory Architecture & Allocation Model

**Zero-Copy Shared Caches, Buffers, and Heap Optimization**

---

## 1. Zero-Copy `SharedTelemetryCache`

The central telemetry state follows a single-writer, multi-reader lock-free pattern:

```rust
pub struct SharedTelemetryCache {
    snapshot: Arc<RwLock<TelemetrySnapshot>>,
    update_count: AtomicU64,
}
```

- **Readers**: Widgets and IPC tasks acquire read locks and copy only primitive fields or clone references.
- **Writers**: `TelemetryService` modifies the snapshot once per 10 ms tick.

---

## 2. RingBuffer Allocation (`ipc_protocol::ring_buffer`)

Telemetry historical graphs in WinUI 3 GUI and TUI are backed by fixed-capacity circular ring buffers (`RingBuffer<T>`):

- **Fixed Allocation**: Pre-allocated array memory; push operations overwrite oldest items without triggering heap reallocation (`Vec` resizing).
- **Capacity**: Configured to 60 samples (1 minute of telemetry history at 1s intervals).
