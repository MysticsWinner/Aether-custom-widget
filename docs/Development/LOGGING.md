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

## 3. Pervasive Inter-Service Communication & Connection Logging

Every inter-process communication, connection, and data exchange is instrumented with structured `tracing` contexts:

### 3.1 Named Pipe IPC (`crates/core_engine/src/ipc_server.rs`)
- **Connection Lifecycle**: `INFO` logs on client connection (`client_id`, remote pipe address) and disconnection (`duration_ms`, active connection count).
- **Live Metrics**: Atomic tracking of `TOTAL_IPC_CONNECTIONS`, `ACTIVE_IPC_CONNECTIONS`, and `TOTAL_COMMANDS_DISPATCHED`.
- **Command Dispatch**: `DEBUG`/`TRACE` logs measuring command variant, payload length, and dispatch latency in microseconds (`dispatch_latency_us`).
- **Error Handling**: `WARN` on stream EOF, unparseable frames, or pipe communication failures.

### 3.2 Shared Memory Ring Buffer (`crates/ipc_protocol/src/ring_buffer.rs`)
- **Write Sequence**: `TRACE` logs on seqlock start and commit (`seq`, payload size).
- **Torn Read Detection**: `WARN` logs when reader detects concurrently modified memory, triggering spin-retry.
- **Reader Catchup & Overwrite**: `WARN` logs on ring buffer wraparound when reader cursor is overwritten.
- **Writer Crash Recovery**: `ERROR` logs if writer fails mid-write or watchdog heartbeat expires.

### 3.3 Event Bus (`crates/core_engine/src/event_bus.rs`)
- **Event Dispatch**: `DEBUG` logs recording event variant, subscriber count, and delivery status.
- **Lag / Drop Isolation**: `TRACE` logs for dropped ephemeral telemetry ticks when subscribers lag.
- **Replay & Gap Detection**: `WARN` logs with full context (`requested_seq`, `oldest_seq`) when subscriber experiences buffer overflow.

### 3.4 Telemetry Service & Providers (`crates/system_providers`)
- **Microsecond Tick Latency**: `TRACE` logs measuring tick duration (`tick_duration_us`).
- **Provider Fault Isolation**: `WARN` logs when individual hardware providers fail or time out, preventing daemon crashes.
- **Sensor Fallbacks**: `DEBUG` logs when fallback sensors or authentic hardware queries activate.

---

## 4. Diagnostics & Fallbacks

- **Live Diagnostics Query**: The `GetDiagnostics` IPC command returns live connection metrics, command throughput, shared memory sequence state, and subsystem health.
- **TUI Dashboard Fallback**: `dashboard_tui` displays an offline cached badge with retry counters if the core engine pipe is temporarily interrupted.
- **Subsystem Degradation**: If a subsystem fails or experiences unrecoverable errors, the host orchestrator automatically logs the failure and broadcasts `CoreEvent::SubsystemSignal { signal: "STATE_DEGRADED" }`.

---

## 5. Client Log Receivers

- **WinUI 3 GUI Dashboard**: `LogCollectorService.cs` captures engine output and displays interactive log streams filtered by severity level (`INFO`, `WARN`, `ERROR`).
- **Ratatui TUI Dashboard**: Dedicated log viewer panel rendered at bottom of CLI dashboard interface, plus `'d'` key for live diagnostics overview.

