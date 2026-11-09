# Memory Allocation Analysis & Zero-Allocation Hot Loops

This document details the memory management strategies enforcing a total physical working set of **`< 25 MB`** with zero heap allocations during the active rendering tick loop.

---

## 🧠 Memory Management Paradigm

1. **Static Buffer Pre-Allocation**: Direct2D vertex buffers, render command queues, and IPC ring buffers are pre-allocated during `on_load()` initialization.
2. **Zero Heap Allocation in Hot Loop**: No `String`, `Vec`, or dynamic heap objects are allocated inside `Engine::tick()` or `Direct2DRenderer::draw()`.
3. **Rust Arena & Slab Allocators**: Reusable element pools maintain stable memory addresses, preventing GC latency or heap fragmentation.
4. **Active Garbage Collection & Working-Set Trimming**:
   - `LruResourceCache::collect_garbage()` periodically evicts stale text layouts and render targets.
   - `FrameArena::collect_garbage()` shrinks scratch arenas down to 64 KB baseline during low-load intervals.
   - Embedded Lua VM triggers `collect_garbage()` automatically upon hot-reloading scripts.
   - Win32 `SetProcessWorkingSetSize` unmaps cold private pages back to Windows kernel pool.
5. **Development Mode Acceleration (`--dev`, `AETHER_DEV_MODE=1`)**:
   - Increases GC cadence from 300s to 15s.
   - Enforces `.NET` Large Object Heap (LOH) compaction (`GCLargeObjectHeapCompactionMode.CompactOnce`) in the WinUI 3 dashboard.
   - Executes full blocking generational collections to prevent memory leaks during rapid widget edits and code hot-reloads.

```
[ Pre-allocated Ring Buffer (64KB) ] ---> [ Zero Allocation Tick Loop ] ---> [ Direct2D Render Target ]
                                                  |
                                   (Periodic / DevMode GC Sweep)
                                                  v
                     [ LRU Prune + Lua GC + Win32 Working-Set Trim ]
```

