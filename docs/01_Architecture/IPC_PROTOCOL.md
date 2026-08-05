# Aether — IPC Protocol Specification

**Named Pipe Architecture, JSON Schema, and Diagnostics API**

---

## 1. IPC Server Configuration

- **Pipe Address**: `\\.\pipe\CustomWidgetEngineControlPipe`
- **Transport**: Windows Named Pipe (`tokio::net::windows::named_pipe::ServerOptions`)
- **Protocol**: JSON Line-delimited UTF-8 strings
- **Concurrency**: Asynchronous multi-client listener loop

---

## 2. Command Variant Reference (`ControlCommand`)

All IPC messages sent from clients follow the `ControlCommand` JSON schema:

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
      "net_recv_bytes_per_sec": 10240,
      "net_sent_bytes_per_sec": 5120
    },
    "active_widgets": ["perf_monitor_widget"],
    "subsystem_health": {
      "TelemetrySubsystem": "Healthy",
      "RenderSubsystem": "Healthy"
    }
  }
}
```

### 2.3 Widget Lifecycle Control
- **Load Widget Request**: `{"type": "LoadWidget", "payload": {"manifest_path": "crates/perf_monitor_widget/widget.toml"}}`
- **Unload Widget Request**: `{"type": "UnloadWidget", "payload": {"widget_id": "perf_monitor_widget"}}`

### 2.4 Theme Mode Switch
- **Request**: `{"type": "SetThemeMode", "payload": {"mode": "Dark"}}`

### 2.5 Subsystem Health & Diagnostics
- **Get Health Request**: `{"type": "GetSubsystemHealth"}`
- **Get Diagnostics Request**: `{"type": "GetDiagnostics"}`

---

## 3. Client Implementation Matrix

| Client | Implementation Path | Language | Notes |
|---|---|---|---|
| **WinUI 3 Dashboard** | `src_gui/CustomWidget.Dashboard/IPCClient/NamedPipeClient.cs` | C# (.NET 8) | Async `NamedPipeClientStream` with timeout and auto-reconnect. |
| **Ratatui TUI** | `crates/dashboard_tui/src/main.rs` | Rust | Tokio `ClientOptions` pipe client with live polling loop. |
