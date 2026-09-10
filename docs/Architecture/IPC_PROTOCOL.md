# Aether — IPC Protocol Specification

**Dual-Channel Named Pipe & Shared Memory Ring Buffer IPC Architecture**

---

## 1. Hybrid Dual-Channel Architecture

Aether separates control traffic and high-frequency hardware metrics into two purpose-built channels:

```mermaid
graph LR
    subgraph Host ["Core Engine Host Daemon"]
        PipeServer["Win32 Named Pipe Server"]
        ShmProducer["Shared Memory Ring Buffer Producer"]
    end

    subgraph Clients ["WinUI 3 GUI Dashboard, CLI & AppContainer Plugins"]
        PipeClient["Named Pipe Client"]
        ShmConsumer["Shared Memory Consumer"]
    end

    PipeClient <-->|Control Commands (JSON Text / Async Duplex)| PipeServer
    ShmProducer -->|Zero-Copy Telemetry Stream (MetricPayload)| ShmConsumer
```

### Channel 1: Control Channel (Named Pipes)
- **Pipe Address**: `\\.\pipe\CustomWidgetEngineControlPipe`
- **Transport**: Windows Named Pipe (`tokio::net::windows::named_pipe::ServerOptions`)
- **Protocol**: JSON Line-delimited UTF-8 strings (`\n`)
- **Concurrency**: Asynchronous multi-client listener loop with non-blocking I/O
- **Latency**: `< 8 µs` roundtrip

### Channel 2: High-Frequency Telemetry Channel (Shared Memory)
- **Mechanism**: Win32 File Mapping (`CreateFileMappingW`, `MapViewOfFile`)
- **Address**: `Local\AetherTelemetrySharedMemory`
- **Header**: `ShmHeader` (magic: `0x41455448`, protocol_version: `1`, seqlock sequence, timestamp, producer PID, liveness)
- **Payload**: Fixed C-compatible struct `MetricPayload` (`CPU`, `GPU`, `RAM`, `Network`, displays, audio)
- **Multi-Reader Protocols**:
  - `MultiReaderSnapshotBuffer`: Seqlock double-buffered snapshot enabling multiple concurrent readers (WinUI 3 GUI, ratatui TUI, widgets) with zero lock contention and zero reader-reader interference.
  - `MultiReaderRingBuffer`: Circular buffer with monotonic write sequence and independent per-reader cursors (`MultiReaderCursor`).
  - `SharedMemoryRingBuffer`: Dedicated SPSC lock-free circular queue for 1:1 message streaming.
- **Realistic Performance Profile**: Eliminates Win32 Named Pipe IPC serialization and message-copy overhead during steady state. Note that memory-mapped I/O remains subject to OS paging, page faults, CPU cache line invalidations, and Win32 named event synchronization overhead.

---

## 2. Command Variant Reference (`ControlCommand`)

All IPC messages sent over the Control Channel follow the `ControlCommand` JSON schema:

### 2.1 Ping / Pong
- **Request**: `{"type": "Ping"}`
- **Response**: `{"type": "Pong"}`

### 2.2 Get Status & Telemetry (`GetStatus`)
- **Request**: `{"type": "GetStatus"}`
- **Response Payload**:
```json
{
  "type": "StatusResponse",
  "payload": {
    "status": "Running",
    "uptime_seconds": 1245,
    "metrics": {
      "cpu_pct": 14.2,
      "gpu_pct": 22.5,
      "ram_used_mb": 4096.0,
      "ram_total_mb": 16384.0,
      "ram_free_mb": 12288.0,
      "net_recv_bytes_per_sec": 10240,
      "net_sent_bytes_per_sec": 5120
    },
    "active_widgets": ["perf_monitor_widget", "weather_widget", "network_monitor_widget"],
    "subsystem_health": {
      "TelemetrySubsystem": "Healthy",
      "RenderSubsystem": "Healthy",
      "ThemeEngineSubsystem": "Healthy",
      "PluginSandboxSubsystem": "Healthy"
    },
    "engine_version": "0.6.0"
  }
}
```

### 2.3 Widget Lifecycle Control
- **Load Widget**: `{"type": "LoadWidget", "payload": {"manifest_path": "crates/perf_monitor_widget/widget.toml"}}`
- **Unload Widget**: `{"type": "UnloadWidget", "payload": {"widget_id": "perf_monitor_widget"}}`
- **Reload All**: `{"type": "ReloadAll"}`
- **Set Widget Render Config**: `{"type": "SetWidgetRenderConfig", "payload": {"widget_id": "perf_monitor_widget", "config_json": "{...}"}}`
- **Get Widget Render Config**: `{"type": "GetWidgetRenderConfig", "payload": {"widget_id": "perf_monitor_widget"}}`

### 2.4 Theme & Design Token Control
- **Set Theme Mode**: `{"type": "SetThemeMode", "payload": {"mode": "Dark"}}`
- **Apply Theme Tokens**: `{"type": "ApplyThemeTokens", "payload": {"theme_id": "theme.cyberpunk.neon"}}`

### 2.5 Desktop Profiles & AI Synthesis
- **Switch Profile**: `{"type": "SwitchProfile", "payload": {"profile_name": "Gaming"}}`
- **Rollback Profile**: `{"type": "RollbackProfile"}`
- **Synthesize AI Layout**: `{"type": "SynthesizeAiLayout", "payload": {"prompt": "4K triple monitor dev layout"}}`
- **Synthesize AI Theme**: `{"type": "SynthesizeAiTheme", "payload": {"prompt": "cyberpunk neon glassmorphism"}}`

### 2.6 Subsystem Health & Diagnostics
- **Get Health**: `{"type": "GetSubsystemHealth"}`
- **Get Diagnostics**: `{"type": "GetDiagnostics"}`

### 2.7 Package Manager & Marketplace
- **Search Marketplace**: `{"SearchMarketplace": {"query": "monitoring", "category": "all"}}` (supports optional `category` filter, with `None` fallback)

### 2.8 Security Audit Logs & Tamper-Evident Verification
- **Get Audit Logs**: `{"type": "GetAuditLogs"}`
- **Verify Audit Chain**: `{"type": "VerifyAuditChain"}`
- **Get Quarantine List**: `{"type": "GetQuarantineList"}`
- **Release Quarantine**: `{"ReleaseQuarantine": {"widget_id": "clock_w"}}`

---

## 3. Client Implementation Matrix

| Client | Implementation Path | Language | Notes |
|:---|:---|:---|:---|
| **WinUI 3 Dashboard** | `src_gui/CustomWidget.Dashboard/Services/AetherIpcService.cs` | C# (.NET 8) | Async `NamedPipeClientStream` with chunked memory-stream reading (`while > 0`), timeout, exponential backoff retries, power/visibility-aware adaptive polling (500ms on AC, 2000ms on battery, 5000ms minimized), and thread-safe volatile connection tracking. |
| **Ratatui TUI** | `crates/dashboard_tui/src/main.rs` | Rust | Tokio `ClientOptions` pipe client with live polling loop. |
| **CLI Tools** | `crates/dev_tools/src/ipc.rs` | Rust | Low-latency synchronous and async pipe helpers. |

