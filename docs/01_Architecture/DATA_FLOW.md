# Aether — Data Flow Architecture

**End-to-End Metric and Command Pipelines**

---

## 1. End-to-End Telemetry Pipeline

Hardware metrics flow through a zero-copy, single-writer pipeline from kernel APIs down to UI dashboard renderers:

```mermaid
sequenceDiagram
    autonumber
    participant Win32 as Win32 Kernel APIs
    participant Providers as TelemetryService / Providers
    participant Cache as SharedTelemetryCache
    participant Bus as Core Event Bus
    participant Widget as PerfMonitorWidget
    participant IPC as Named Pipe Server
    participant GUI as WinUI 3 Dashboard / TUI

    loop Every 10ms Engine Tick
        Providers->>Win32: GetSystemTimes() & GlobalMemoryStatusEx()
        Win32-->>Providers: Kernel CPU ticks & Memory bytes
        Providers->>Cache: update_snapshot(TelemetrySnapshot)
        Cache-->>Bus: CoreEvent::TelemetryTick
    end

    loop Every Widget Update (500ms)
        Widget->>Cache: get_snapshot()
        Cache-->>Widget: Read-only TelemetrySnapshot reference
        Widget->>Widget: Compute DrawCommand batch (FillRect, Text)
    end

    loop Every IPC Poll (200ms)
        GUI->>IPC: ControlCommand::GetStatus
        IPC->>Cache: get_snapshot()
        Cache-->>IPC: MetricPayload
        IPC-->>GUI: JSON Response (CPU%, RAM used/total, health)
        GUI->>GUI: Update WinUI 3 Gauge Cards & Chart RingBuffer
    end
```

---

## 2. Command & Control Pipeline

Control commands flow bi-directionally between IPC clients (WinUI 3 GUI or Ratatui TUI) and the Rust engine daemon:

```mermaid
sequenceDiagram
    autonumber
    participant Client as WinUI 3 App / TUI
    participant Pipe as Named Pipe PipeServer
    participant Router as Command Router
    participant Engine as Engine State Machine

    Client->>Pipe: Write JSON: {"type": "SetThemeMode", "mode": "Dark"}
    Pipe->>Router: Deserialize ControlCommand::SetThemeMode
    Router->>Engine: Mutate ThemeState & trigger ThemeSubsystem
    Engine-->>Pipe: Serialize Response: {"status": "Ok", "message": "Theme changed to Dark"}
    Pipe-->>Client: Receive JSON acknowledgment
```

---

## 3. Data Schema Standards

### 3.1 Telemetry Snapshot Schema (`system_providers::SharedTelemetryCache`)
```rust
pub struct TelemetrySnapshot {
    pub timestamp_ms: u64,
    pub cpu_usage_pct: f32,
    pub gpu_usage_pct: f32,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub net_recv_bytes_per_sec: u64,
    pub net_sent_bytes_per_sec: u64,
    pub custom_metrics: HashMap<String, f64>,
}
```

### 3.2 IPC Wire Message Schema (`ipc_protocol::messages`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ControlCommand {
    Ping,
    Pong,
    GetStatus,
    LoadWidget { manifest_path: String },
    UnloadWidget { widget_id: String },
    SetThemeMode { mode: String },
    ReloadAll,
    GetSubsystemHealth,
    GetDiagnostics,
    ToggleDesktopWidget,
    SetWidgetPosition { widget_id: String, x: i32, y: i32 },
    SetWidgetLock { widget_id: String, locked: bool },
    ToggleWidgetLock { widget_id: String },
}
```

