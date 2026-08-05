# Aether — Core Optimization Rules

**Mandatory Performance Optimization Guidelines**

---

## Mandatory Optimization Guidelines

1. **No Allocations in 10ms Tick Loop**: Avoid `String::from` or `Vec::new()` calls inside `Subsystem::tick()`. Re-use pre-allocated buffers.
2. **Minimizing Lock Contention**: Acquire `RwLock` read locks only for the exact duration of data extraction. Never perform async IO while holding lock guards.
3. **Dirty Rect Skipping**: If `dirty_regions.is_empty()`, skip draw call generation and GPU composition entirely.
4. **IPC Message Batching**: Batch metric payload updates (send combined CPU/GPU/RAM JSON rather than separate streams).
