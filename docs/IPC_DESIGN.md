# IPC Design Specification (Named Pipes & Shared Memory)

This document describes the high-throughput inter-process communication (IPC) system connecting the Core Engine Host Daemon, WinUI 3 Dashboard, and sandboxed plugin processes.

---

## 🔀 Hybrid Dual-Channel IPC Architecture

```mermaid
graph LR
    subgraph Host ["Core Engine Host Daemon"]
        PipeServer["Win32 Named Pipe Server"]
        ShmProducer["Shared Memory Ring Buffer Producer"]
    end

    subgraph Clients ["WinUI 3 GUI & AppContainer Plugins"]
        PipeClient["Named Pipe Client"]
        ShmConsumer["Shared Memory Consumer"]
    end

    PipeClient <-->|Control Commands (JSON)| PipeServer
    ShmProducer -->|Zero-Copy Telemetry Stream| ShmConsumer
```

### 1. Control Channel: Win32 Named Pipes
- **Pipe Name**: `\\\\.\\pipe\\CustomWidgetEngineControlPipe`
- **Protocol**: Asynchronous duplex JSON messages (`ControlCommand::SetThemeMode`, `ControlCommand::LoadWidget`, `ControlCommand::ReloadAll`).
- **Latency**: `< 8 µs` roundtrip.

### 2. High-Frequency Telemetry Channel: Shared Memory Ring Buffer
- **Mechanism**: Win32 File Mapping (`CreateFileMappingW`, `MapViewOfFile`).
- **Payload**: Fixed `MetricPayload` C-struct (CPU, RAM, GPU, Net throughput) written by single telemetry producer thread. Consumers read zero-copy memory directly without IPC context switches.
