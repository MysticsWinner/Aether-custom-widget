# Aether — Structured Logging System

**Diagnostics, Telemetry Tracing, and Log Collectors**

---

## 1. Tracing Subsystem Architecture

Aether uses `tracing` and `tracing-subscriber` across all Rust engine components for structured logging:

```rust
// Initialized in core_engine::main()
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(true)
    .with_thread_ids(true)
    .init();
```

---

## 2. Standardized Log Levels

| Level | Usage Guidance | Example Event |
|---|---|---|
| `ERROR` | Severe failures requiring immediate intervention or subsystem shutdown. | IPC Pipe Server bind failure, memory allocation failure. |
| `WARN` | Non-fatal abnormalities or degraded operations. | Telemetry sample dropped, manifest parsed with default fallbacks. |
| `INFO` | Major engine lifecycle events and system status changes. | Subsystems initialized, widget loaded, IPC client connected. |
| `DEBUG` | Fine-grained subsystem tick details and metrics serialization. | Telemetry cache tick timing, IPC payload byte count. |
| `TRACE` | Verbose frame-by-frame dirty region calculations. | Rect union calculations, dirty pixel bounds. |

---

## 3. Client Log Receivers

- **WinUI 3 GUI Dashboard**: `LogCollectorService.cs` captures engine output and displays interactive log streams filtered by severity level (`INFO`, `WARN`, `ERROR`).
- **Ratatui TUI Dashboard**: Dedicated log viewer panel rendered at bottom of CLI dashboard interface.
