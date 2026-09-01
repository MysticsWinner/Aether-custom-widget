# Aether — Error Handling Strategy

**Error Taxonomy, Propagation, and Fault Resilience**

---

## 1. Error Handling Philosophy

Aether adopts explicit error propagation rules:
- **No Uncaught Panics**: Engine tick tasks must never panic; failures must be caught, logged, and surfaced via `SubsystemHealth::Degraded` or `Unhealthy`.
- **Structured Error Types**: Domain crates define explicit error enums using `thiserror`. Application entry points and high-level orchestrators use `anyhow::Result`.

---

## 2. Error Taxonomy (`thiserror` Pattern)

Subsystem crates implement custom error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Subsystem '{0}' failed to initialize: {1}")]
    SubsystemInitFailed(String, String),

    #[error("IPC Named Pipe server error: {0}")]
    IpcServerError(String),

    #[error("Widget manifest invalid at '{path}': {reason}")]
    InvalidManifest { path: String, reason: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## 3. Subsystem Health Tracking

When errors occur during engine ticks, the subsystem updates its internal `SubsystemHealth` state:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubsystemHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}
```

The `SubsystemManager` tracks subsystem health status. If a subsystem enters `Unhealthy`, the engine logs diagnostics and executes localized recovery actions via `FaultDiagnostics`.
