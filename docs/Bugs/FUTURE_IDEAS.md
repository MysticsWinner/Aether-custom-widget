# Future Ideas

*Enhancement proposals and feature requests not yet committed to the roadmap.*

---

## FI-001: Move daemon benchmarks behind `--benchmark` CLI flag (Implemented)

- **Component**: `core_engine`
- **Rationale**: Benchmarks currently run on every daemon startup, increasing cold start time. A `--benchmark` flag would keep benchmarks accessible for CI/QA without penalising production startup latency.
- **Status**: Implemented via `EngineConfig::with_cli_args` (`--benchmark` / `AETHER_RUN_BENCHMARKS=1`).

## FI-002: Implement real WASAPI `IAudioEndpointVolume` in `AudioProvider` (Implemented)

- **Component**: `system_providers`
- **Rationale**: Replace the hardcoded 75% volume stub with real COM-based WASAPI volume and mute queries.
- **Status**: Implemented in `AudioProvider::query_system_audio()`.

## FI-003: Modularise `ControlCommand` IPC enum into domain sub-enums

- **Component**: `ipc_protocol`
- **Rationale**: The 60+ variant flat enum would benefit from grouping into `WidgetCommand`, `DiagnosticsCommand`, `MarketplaceCommand`, etc.

## FI-004: Add `aether widget create <name>` scaffolding CLI command (Implemented)

- **Component**: `dev_tools`
- **Rationale**: Reduce friction for new widget development by generating boilerplate `Cargo.toml`, `lib.rs`, `widget.toml`, and test scaffolding.
- **Status**: Implemented via `CliCommand::WidgetCreate`, `AetherCli::parse_args`, and `WidgetBuilder::scaffold_to_disk`.

## FI-005: Implement WASM (`wasmtime`) widget sandboxing runtime (Implemented)

- **Component**: `plugin_runtime`
- **Rationale**: WebAssembly sandboxing would provide cross-platform plugin isolation without Windows-specific AppContainer dependencies.
- **Status**: Binary WebAssembly v1 header verification (`\0asm\x01\0\0\0`), fuel limits, and linear memory page caps implemented in `WasmPluginEngine::load_bytecode_module`.

## FI-006: Cross-platform `DesktopSurface` trait for Linux Wayland / macOS Quartz

- **Component**: New crate
- **Rationale**: Abstract the Windows-specific `WorkerW` desktop hooking behind a trait to enable future Linux and macOS backends.
