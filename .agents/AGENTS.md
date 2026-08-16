Read these documents before writing code.

Priority order

1. AGENTS.md

2. docs/Architecture/

3. docs/Development/

4. docs/Testing/

5. docs/Security/

6. docs/Performance/

7. docs/API/

If two documents conflict,

Architecture wins.

Never invent architecture.

Never skip phases.

Never replace real Windows APIs with simulated code.

Every PR requires

Tests

Benchmarks

Documentation

Security review

Performance analysis

Run cargo check

Run cargo test

No task is complete until tests pass.

After every prompt, update the documents in docs/ accordingly, dont remove content unless it is neccessary. For example: "Feature: show CPU metrics." is there and only a partial update has been done like "Feature: Show CPU usage, temperature, cores ..." -> then edit the line to "Feature: Show CPU usage, temperature ..." and dont remove the "..." if it is there, instead add the new feature to it. Make sure that the docs are always up to date with the current implementation.

## Workflow & Process Governance
1. **Never skip phases**: Follow the phased architecture roadmap strictly.
2. **Architecture Prerequisite**: Never implement features whose architecture has not been finalized.
3. **Comprehensive Deliverables**: Every feature/pull request must include:
   - Unit tests
   - Benchmarks
   - Documentation
   - Logging
   - Error handling
   - Performance analysis
   - Security review

## System & Software Architecture
4. **Interface Isolation**: Every subsystem must expose interfaces (e.g. traits/abstract APIs) instead of concrete implementations.
5. **Composition**: Prefer composition over inheritance.
6. **State & Immutability**:
   - Avoid global state.
   - Prefer immutable data structures and zero-side-effect functions.
7. **Dependency & Build Constraints**:
   - Never optimize prematurely.
   - Keep dependencies minimal and audit regularly.
   - Everything must compile cleanly on Windows 11 (`x86_64` & `ARM64`).

## Documentation, Trade-Offs & Quality
8. **Trade-Off Analysis**: Every design decision must explicitly outline alternatives considered and rationale.
9. **Architectural Visualizations**: Generate visual diagrams (Mermaid) whenever architecture or data flows change.
10. **Production Quality**: Maintain production-grade quality, strict linting, and error handling at every step.

---

## Mandatory Testing Protocol (applies to EVERY prompt)

> **Rule**: Regardless of the size or nature of the request, every code change MUST include tests
> that are verified to pass before the task is considered complete. No exceptions.
 For every new code, if it has any functional value and requires a test to be created, create the test following the testing methodology.

### Required steps for every prompt:
1. **Write tests** alongside (or before) the implementation -- never skip them.
2. **Run `cargo test --workspace`** to verify ALL Rust backend tests pass, not just new ones.
3. **Run `cargo check --workspace`** to confirm zero Rust compilation errors.
4. **Run `dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj`** to confirm zero C# WinUI 3 build errors.
5. **Run `dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj`** to verify GUI ViewModel & IPC tests pass.
6. **Report the test count** in every response (e.g. "268 Rust tests + 10 C# GUI tests pass").
7. **Do not mark a task done** until `cargo test` and `dotnet test` exit with code 0.

### Minimum test coverage per change type:

| Change Type | Minimum Tests Required |
|---|---|
| New public function / method | >= 1 unit test per function |
| New struct / trait impl | >= 1 lifecycle / happy-path test |
| New crate | >= 3 tests: normal path, edge case, error case |
| Bug fix | >= 1 regression test proving the bug is gone |
| Refactor | All pre-existing tests must still pass |
| Config / manifest change | >= 1 parse or roundtrip test |

### Test naming convention (Rust):
```rust
#[test]
fn test_<unit_under_test>_<scenario>() { ... }

// Examples:
fn test_cpu_provider_percentage_in_range() { ... }
fn test_perf_widget_lifecycle_mounts_cleanly() { ... }
fn test_ipc_dispatch_get_status_returns_json() { ... }
```

---

## Project Knowledge Base

### Repository Overview
- **Name**: Aether -- Next-Gen Windows Desktop Customization Platform
- **Language**: Rust (backend / engine) + C# / WinUI 3 (GUI dashboard)
- **Target OS**: Windows 11, x86_64 & ARM64
- **Phase**: 16 (Production Release Candidate — Diagnostics & Integration)
- **Root**: `d:\Code\Aether-custom-widget\`
- **Workspace manifest**: `Cargo.toml` at repo root
- **Current test count**: 121 -- run `cargo test --workspace` to verify. This number is subjected to change but only expected to increase over the time.


### Core Architecture Principle -- "Collect Once, Publish Everywhere"
- A single `TelemetrySubsystem` tick collects all hardware metrics once per 10 ms engine cycle.
- Widgets read exclusively from `SharedTelemetryCache` -- zero repeated OS API calls per widget.
- Widgets emit `DrawCommand` batches; the render host composites via DirectComposition / Direct2D.

### Crate Map

| Crate | Path | Responsibility |
|---|---|---|
| `core_engine` | `crates/core_engine` | Tokio async host daemon; subsystem orchestrator; IPC pipe server; event bus |
| `system_providers` | `crates/system_providers` | Hardware collectors (CPU via `GetSystemTimes`, RAM via `GlobalMemoryStatusEx`, GPU simulation) + `SharedTelemetryCache` |
| `widget_sdk` | `crates/widget_sdk` | `WidgetLifecycle` trait, `RenderCanvas`, `BatchRenderCanvas`, `DrawCommand`, animations, events, settings |
| `perf_monitor_widget` | `crates/perf_monitor_widget` | Built-in performance widget -- CPU%, GPU%, RAM used/free with dark glassmorphism card renderer |
| `widget_parser` | `crates/widget_parser` | TOML widget manifest schema (`WidgetManifest`, `WidgetElement`, `LayoutSpec`) |
| `ipc_protocol` | `crates/ipc_protocol` | Shared IPC types: `ControlCommand` enum, `MetricPayload` struct (serde JSON) |
| `plugin_runtime` | `crates/plugin_runtime` | AppContainer sandbox supervisor, API version compatibility checker |
| `layout_engine` | `crates/layout_engine` | Flexbox layout computation |
| `theme_engine` | `crates/theme_engine` | JSON theme schema, hot-reload watcher, token resolver |
| `animation_engine` | `crates/animation_engine` | Easing curves, spring physics, timeline scheduling |
| `lua_runtime` | `crates/lua_runtime` | Lua scripting bridge for widget logic |
| `package_manager` | `crates/package_manager` | npm-style widget installer with Ed25519 signature verification |
| `cloud_sync` | `crates/cloud_sync` | CRDT-based config sync with offline mode |
| `ai_engine` | `crates/ai_engine` | AI layout/theme/widget synthesis, voice commands, workflow automation |
| `production_engine` | `crates/production_engine` | Security audits, stress testing, auto-updater, crash analytics |
| `dashboard_tui` | `crates/dashboard_tui` | ratatui terminal dashboard -- polls IPC pipe, renders animated CPU/GPU/RAM gauges |
| `installer` | `crates/installer` | Windows NSIS-style setup installer |
| `CustomWidget.Dashboard` | `src_gui/CustomWidget.Dashboard` | WinUI 3 C# management dashboard (requires VS2022 + Windows App SDK 1.5) |

### Key Interfaces

#### `WidgetLifecycle` trait -- `widget_sdk/src/lifecycle.rs`
Every widget plugin MUST implement this trait:
- `on_load(&mut self) -> Result<()>` -- allocate resources
- `on_mount(&mut self) -> Result<()>` -- attach to desktop canvas
- `on_update(&mut self, ctx: &TickContext) -> Result<()>` -- per-tick: read cache, emit DrawCommands
- `on_unmount(&mut self) -> Result<()>` -- detach from canvas
- `on_unload(&mut self) -> Result<()>` -- free resources
- `state(&self) -> WidgetState` -- current lifecycle state

#### `RenderCanvas` trait -- `widget_sdk/src/rendering.rs`
- `draw_rect(rect: RectF, color: Color, corner_radius: f32)`
- `draw_text(text: &str, font: &str, size: f32, rect: RectF, color: Color)`
- `commands() -> &[DrawCommand]` -- retrieve the batched draw commands

Use `BatchRenderCanvas::new()` as the concrete implementation in `on_update`.

#### `SharedTelemetryCache` -- `system_providers/src/shared_cache.rs`
- `cache.get_cpu_pct() -> f32` -- latest CPU %
- `cache.get_memory_used_mb() -> f32` -- latest RAM used MB
- `cache.get_snapshot() -> TelemetrySnapshot` -- full snapshot (CPU, GPU, RAM, NET)
- `cache.update_count() -> u64` -- number of telemetry ticks since creation

#### `TelemetrySnapshot` fields -- `system_providers/src/shared_cache.rs`
`timestamp_ms, cpu_usage_pct, gpu_usage_pct, memory_used_mb, memory_total_mb,
net_recv_bytes_per_sec, net_sent_bytes_per_sec, custom_metrics: HashMap<String, f64>`

#### `ControlCommand` IPC enum -- `ipc_protocol/src/messages.rs`
`Ping | Pong | GetStatus | LoadWidget { manifest_path } | UnloadWidget { widget_id } | SetThemeMode { mode } | ReloadAll`

Serialises to/from JSON via serde_json.
IPC pipe address: `\\.\pipe\CustomWidgetEngineControlPipe`

### How to Run the Project

```powershell
# One-command full stack (opens two windows: daemon + TUI dashboard):
.\launch.ps1

# Manual -- Terminal 1 (daemon):
cargo run -p core_engine

# Manual -- Terminal 2 (TUI dashboard, start after daemon is running):
cargo run -p dashboard_tui

# Run all tests:
cargo test --workspace

# Verify compilation:
cargo check --workspace
```

### Adding a New Widget -- Step-by-Step
1. Create `crates/<name>/Cargo.toml` -- depend on `widget_sdk` + `system_providers`.
2. Implement `WidgetLifecycle` -- in `on_update`, call `cache.get_snapshot()` and push to `BatchRenderCanvas`.
3. Create `crates/<name>/widget.toml` -- declare metric bindings (e.g. `binding = "sys.cpu_usage"`).
4. Add crate to workspace `Cargo.toml` `members`.
5. In `core_engine/src/main.rs`, spawn a tokio task that calls `on_update` at your desired interval.
6. **Write >= 3 tests** covering lifecycle, renderer output, and error/edge cases.
7. Run `cargo test --workspace` -- all tests must pass before the feature is considered complete.

### Key Workspace Dependencies
- `tokio 1.38 (features = ["full"])` -- async runtime
- `tracing` + `tracing-subscriber` -- structured logging
- `serde` + `serde_json` -- IPC serialization
- `anyhow` + `thiserror` -- error handling
- `windows 0.58` -- Win32 APIs (Direct2D, DirectComposition, SystemInformation, Threading, Memory)
- `async-trait 0.1` -- async fn in traits
- `ratatui 0.28` + `crossterm 0.28` -- TUI dashboard (dashboard_tui crate only)
