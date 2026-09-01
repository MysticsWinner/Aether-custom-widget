# Aether — Architecture Decision Records (ADRs)

**Summary of Key Engineering Trade-Offs and Rationale**

---

## ADR 001: Language Selection — Rust Core Engine Backend

- **Status**: Accepted & Implemented
- **Context**: Need a low-latency, memory-safe engine capable of 10 ms tick loops, hardware metric querying, zero-copy telemetry sharing, and low CPU idle overhead.
- **Decision**: Select **Rust (2021 Edition)** with Tokio async runtime.
- **Alternatives Considered**: C++20 (higher memory vulnerability risk), Go (GC pauses disrupt 10ms tick cadence).
- **Consequences**: Exceptional performance, zero data race guarantees, minimal CPU/memory footprint (<15 MB RAM idle).

---

## ADR 002: GUI Framework — WinUI 3 (C# / .NET 8)

- **Status**: Accepted & Implemented
- **Context**: Need a native Windows 11 desktop application shell supporting Fluent Design, Mica backdrops, custom dark theme controls, and responsive UI layout.
- **Decision**: Select **WinUI 3 (Windows App SDK 2.2)** in C# .NET 8.
- **Alternatives Considered**: WPF / WinForms (legacy visual styling), Electron / Tauri (web tech overhead).
- **Consequences**: Premium visual experience native to Windows 11; decoupled process isolation from Rust daemon via Named Pipes.

---

## ADR 003: IPC Transport — Windows Named Pipes

- **Status**: Accepted & Implemented
- **Context**: Decoupled process model requires low-latency bi-directional messaging between Rust core engine and GUI/TUI clients.
- **Decision**: Use Windows **Named Pipe** (`\\.\pipe\CustomWidgetEngineControlPipe`) sending line-delimited JSON (`ControlCommand`).
- **Alternatives Considered**: gRPC / HTTP REST (web server overhead), Shared Memory (complex synchronization for control commands).
- **Consequences**: Sub-millisecond IPC dispatch, native Windows pipe security features, zero external web server dependencies.

---

## ADR 004: Manifest Schema Format — TOML

- **Status**: Accepted & Implemented
- **Context**: Need a human-readable manifest schema format for defining widget metadata, telemetry bindings, layout constraints, and permissions.
- **Decision**: Select **TOML** (`widget.toml`).
- **Alternatives Considered**: JSON (lack of comments, rigid syntax), YAML (indentation error risks).
- **Consequences**: Standardized format familiar to Rust ecosystem, easy file parsing via `serde` + `toml` crate.
