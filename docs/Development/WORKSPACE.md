# Workspace Layout & Crate Map

**Inventory and Responsibilities of all 33 Member Crates in the Aether Workspace**

---

## 1. Workspace Crate Map (33 Member Crates + Test Harness)

| Crate | Responsibility | Path | Test Count |
|:---|:---|:---|:---|
| `core_engine` | Async daemon host, subsystem orchestrator, Direct2D renderer, particles, virtual desktops | `crates/core_engine` | 74 tests |
| `system_providers` | Hardware collectors (CPU, RAM, GPU, WASAPI audio, crypto, network) & `SharedTelemetryCache` | `crates/system_providers` | 22 tests |
| `widget_sdk` | Standardized 6-pillar widget API, `WidgetLifecycle`, `BatchRenderCanvas`, SVG, FrameArena | `crates/widget_sdk` | 28 tests |
| `weather_particles_widget` | Built-in ambient weather & atmospheric Direct2D physical particle simulation widget | `crates/weather_particles_widget` | 2 tests |
| `crypto_stocks_widget` | Built-in real-time cryptocurrency & stock index market ticker widget | `crates/crypto_stocks_widget` | 2 tests |
| `dock_launcher_widget` | Built-in dynamic desktop dock & app launcher with magnification physics and hit testing | `crates/dock_launcher_widget` | 3 tests |
| `hardware_pro_widget` | Built-in dedicated GPU VRAM, 3D/Video load, CPU topology matrix & spline graph widget | `crates/hardware_pro_widget` | 2 tests |
| `audio_visualizer_widget` | Built-in WASAPI 16-band audio FFT visualizer & SMTC media player controller widget | `crates/audio_visualizer_widget` | 3 tests |
| `perf_monitor_widget` | Built-in glassmorphic hardware performance monitor widget | `crates/perf_monitor_widget` | 6 tests |
| `weather_widget` | Built-in weather forecast and city monitoring widget | `crates/weather_widget` | 3 tests |
| `network_monitor_widget` | Built-in network adapter throughput & bandwidth monitor widget | `crates/network_monitor_widget` | 3 tests |
| `ai_assistant_widget` | Natural language AI desktop helper card widget | `crates/ai_assistant_widget` | 3 tests |
| `widget_parser` | TOML manifest schema (`WidgetManifest`, `LayoutSpec`, `PermissionManifest`) | `crates/widget_parser` | 3 tests |
| `ipc_protocol` | Dual-channel IPC types (`ControlCommand`, `MetricPayload`, lock-free shared memory ring buffer) | `crates/ipc_protocol` | 9 tests |
| `plugin_runtime` | AppContainer sandbox supervisor, Wasm linear memory engine & JobObject limits | `crates/plugin_runtime` | 9 tests |
| `layout_engine` | Flexbox layout engine powered by `taffy` | `crates/layout_engine` | 4 tests |
| `theme_engine` | 12-category design token system, JSON theme parser, hot-swapper & MaterialEngine | `crates/theme_engine` | 13 tests |
| `animation_engine` | Spring physics, cubic bezier easing curves & timeline scheduling | `crates/animation_engine` | 1 test |
| `lua_runtime` | Sandboxed Lua 5.4 scripting bridge (`mlua`) with live state-preserving HCR | `crates/lua_runtime` | 4 tests |
| `package_manager` | npm-style `.cwp` package installer & Ed25519 cryptographic verifier | `crates/package_manager` | 5 tests |
| `cloud_sync` | CRDT state-based config synchronization with AES-256-GCM encryption | `crates/cloud_sync` | 6 tests |
| `ai_engine` | 6-pillar AI synthesizer (layout, theme, widget, voice intent parser, workflow automation) | `crates/ai_engine` | 10 tests |
| `production_engine` | Security auditor, stress testing harness, crash analytics & auto-updater | `crates/production_engine` | 7 tests |
| `installer` | Windows local setup wizard and binary packaging tool (`AetherSetup.exe`) | `crates/installer` | 5 tests |
| `dashboard_tui` | Animated Ratatui terminal dashboard with live gauges | `crates/dashboard_tui` | 1 test |
| `recovery_manager` | Crash recovery supervisor & Safe Mode sentinel | `crates/recovery_manager` | 6 tests |
| `config_manager` | Transactional atomic configuration store with 5-generation rolling backups & profile manager | `crates/config_manager` | 11 tests |
| `capability_broker` | Sandboxing capability token grant store & `WidgetFirewall` | `crates/capability_broker` | 7 tests |
| `watchdog` | Heartbeat supervisor daemon | `crates/watchdog` | 2 tests |
| `event_recorder` | Time-travel event stream recorder & replayer | `crates/event_recorder` | 2 tests |
| `observability` | Prometheus metrics exporter, ETW tracing provider, minidump & flight recorder | `crates/observability` | 6 tests |
| `dev_tools` | File watcher hot-reloader, DOM inspector & `aether` CLI | `crates/dev_tools` | 7 tests |
| `enterprise` | Group Policy (GPO) engine & SHA-256 tamper-evident audit logger | `crates/enterprise` | 4 tests |
| `tests/` | Workspace integration, interface, system, and master release audit test suite | `tests/` | 31 tests |

---

## 2. Additional Project Components

| Component | Path | Responsibility |
|:---|:---|:---|
| **WinUI 3 GUI Dashboard** | `src_gui/CustomWidget.Dashboard` | C# .NET 8 WinUI 3 management dashboard app (12 pages, MVVM architecture, 6 services, 30 tests) |
| **C# SDK Bindings** | `bindings/csharp/CustomWidget.SDK` | C# widget interface (`IWidget`) and MVVM models |
| **TypeScript SDK** | `bindings/typescript/custom-widget-sdk` | TypeScript declarations (`index.d.ts`) |
| **Win32 Hook DLLs** | `native/win32_hooks` | Native C++ WorkerW / Progman desktop shell attachment hooks |
