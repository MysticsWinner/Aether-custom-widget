# Future Ideas

*Enhancement proposals and feature requests not yet committed to the roadmap.*

---

## FI-001: Move daemon benchmarks behind `--benchmark` CLI flag

- **Component**: `core_engine`
- **Rationale**: Benchmarks currently run on every daemon startup, increasing cold start time. A `--benchmark` flag would keep benchmarks accessible for CI/QA without penalising production startup latency.

## FI-002: Implement real WASAPI `IAudioEndpointVolume` in `AudioProvider`

- **Component**: `system_providers`
- **Rationale**: Replace the hardcoded 75% volume stub with real COM-based WASAPI volume and mute queries.

## FI-003: Modularise `ControlCommand` IPC enum into domain sub-enums

- **Component**: `ipc_protocol`
- **Rationale**: The 60+ variant flat enum would benefit from grouping into `WidgetCommand`, `DiagnosticsCommand`, `MarketplaceCommand`, etc.

## FI-004: Add `aether widget create <name>` scaffolding CLI command

- **Component**: `dev_tools`
- **Rationale**: Reduce friction for new widget development by generating boilerplate `Cargo.toml`, `lib.rs`, `widget.toml`, and test scaffolding.

## FI-005: Implement WASM (`wasmtime`) widget sandboxing runtime

- **Component**: `plugin_runtime`
- **Rationale**: WebAssembly sandboxing would provide cross-platform plugin isolation without Windows-specific AppContainer dependencies.

## FI-006: Cross-platform `DesktopSurface` trait for Linux Wayland / macOS Quartz

- **Component**: New crate
- **Rationale**: Abstract the Windows-specific `WorkerW` desktop hooking behind a trait to enable future Linux and macOS backends.
