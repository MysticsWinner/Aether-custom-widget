# Aether — Integration Testing Architecture

**IPC Client/Server End-to-End Integration Validation**

---

## 1. IPC Integration Testing Harness

Integration testing validates the bi-directional communication flow between client applications (WinUI 3 GUI and Ratatui TUI) and the Named Pipe IPC server:

```mermaid
sequenceDiagram
    autonumber
    participant TestHarness as Integration Test Harness
    participant Server as Named Pipe Server (crates/core_engine)
    participant Engine as Engine State Machine

    TestHarness->>Server: Connect NamedPipeClientStream
    Server-->>TestHarness: Pipe Connection Established
    TestHarness->>Server: Write JSON: {"type": "Ping"}
    Server-->>TestHarness: Read JSON: {"type": "Pong"}
    TestHarness->>Server: Write JSON: {"type": "GetStatus"}
    Server->>Engine: Query status & telemetry snapshot
    Engine-->>Server: TelemetrySnapshot (CPU, RAM, Subsystem health)
    Server-->>TestHarness: Read JSON: {"type": "StatusResponse", "payload": {...}}
    TestHarness->>TestHarness: Assert payload.cpu_pct >= 0.0 AND payload.ram_total_mb > 0.0
```

---

## 2. Integration Test Scenarios

| Scenario ID | Test Objectives | Verified Assertions |
|---|---|---|
| **INT-IPC-01** | Named Pipe Connection & Ping | Server responds with `Pong` within 10 ms. |
| **INT-IPC-02** | Live Telemetry Streaming | `GetStatus` returns non-zero total RAM and active subsystem status list. |
| **INT-IPC-03** | Multi-Client Pipe Concurrent Access | Concurrent connections from WinUI GUI and Ratatui TUI proceed without pipe lock contention. |
| **INT-IPC-04** | Theme Swapped Signal Propagation | `SetThemeMode` command updates engine state and publishes `CoreEvent::ThemeChanged` to event bus. |
| **INT-SUB-01** | Engine Graceful Shutdown | Requesting `stop()` triggers reverse-order subsystem shutdown cleanly within timeout budget. |
