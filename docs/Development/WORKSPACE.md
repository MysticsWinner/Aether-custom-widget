# Workspace Layout & Crate Map

**Inventory and Responsibilities of all 28 Member Crates in the Aether Workspace**

---

## 1. Workspace Crate Map (28 Member Crates + Test Harness)

| Crate | Responsibility | Path | Test Count |
|:---|:---|:---|:---|
| `core_engine` | Async daemon host, subsystem orchestrator, IPC pipe server, Direct2D renderer | `crates/core_engine` | 58 tests |
| `system_providers` | Hardware telemetry collectors (CPU via `GetSystemTimes`, RAM via `GlobalMemoryStatusEx`, GPU, Net) & `SharedTelemetryCache` | `crates/system_providers` | 11 tests |
| `widget_sdk` | Standardized 6-pillar widget API, `WidgetLifecycle`, `BatchRenderCanvas`, reactive signals | `crates/widget_sdk` | 22 tests |
| `perf_monitor_widget` | Built-in glassmorphic hardware performance monitor widget | `crates/perf_monitor_widget` | 6 tests |
| `weather_widget` | Built-in weather forecast and city monitoring widget | `crates/weather_widget` | 3 tests |
| `network_monitor_widget` | Built-in network adapter throughput & bandwidth monitor widget | `crates/network_monitor_widget` | 3 tests |
| `ai_assistant_widget` | Natural language AI desktop helper card widget | `crates/ai_assistant_widget` | 3 tests |
| `widget_parser` | TOML manifest schema (`WidgetManifest`, `LayoutSpec`, `PermissionManifest`) | `crates/widget_parser` | 3 tests |
| `ipc_protocol` | Dual-channel IPC types (`ControlCommand`, `MetricPayload`, shared memory ring buffer) | `crates/ipc_protocol` | 8 tests |
| `plugin_runtime` | AppContainer sandbox supervisor, low-integrity process launcher & JobObject limits | `crates/plugin_runtime` | 7 tests |
| `layout_engine` | Flexbox layout engine powered by `taffy` | `crates/layout_engine` | 4 tests |
| `theme_engine` | 12-category design token system, JSON theme parser, hot-swapper & MaterialEngine | `crates/theme_engine` | 13 tests |
| `animation_engine` | Spring physics, cubic bezier easing curves & timeline scheduling | `crates/animation_engine` | 1 test |
| `lua_runtime` | Sandboxed Lua 5.4 scripting bridge (`mlua`) | `crates/lua_runtime` | 3 tests |
| `package_manager` | npm-style `.cwp` package installer & Ed25519 cryptographic verifier | `crates/package_manager` | 5 tests |
| `cloud_sync` | CRDT state-based config synchronization with AES-256-GCM encryption | `crates/cloud_sync` | 6 tests |
| `ai_engine` | 6-pillar AI synthesizer (layout, theme, widget, voice intent parser, workflow automation) | `crates/ai_engine` | 10 tests |
| `production_engine` | Security auditor, stress testing harness, crash analytics & auto-updater | `crates/production_engine` | 7 tests |
| `installer` | Windows local setup wizard and binary packaging tool (`AetherSetup.exe`) | `crates/installer` | 5 tests |
| `dashboard_tui` | Animated Ratatui terminal dashboard with live gauges | `crates/dashboard_tui` | 1 test |
| `recovery_manager` | Crash recovery supervisor & Safe Mode sentinel | `crates/recovery_manager` | 6 tests |
| `config_manager` | Transactional atomic configuration store with 5-generation rolling backups & profile manager | `crates/config_manager` | 9 tests |
| `capability_broker` | Sandboxing capability token grant store & `WidgetFirewall` | `crates/capability_broker` | 7 tests |
| `watchdog` | Heartbeat supervisor daemon | `crates/watchdog` | 2 tests |
| `event_recorder` | Time-travel event stream recorder & replayer | `crates/event_recorder` | 2 tests |
| `observability` | Prometheus metrics exporter, ETW tracing provider & minidump writer | `crates/observability` | 4 tests |
| `dev_tools` | File watcher hot-reloader & Chrome DOM inspector | `crates/dev_tools` | 5 tests |
| `enterprise` | Group Policy (GPO) engine & SHA-256 tamper-evident audit logger | `crates/enterprise` | 4 tests |
| `tests/` | Workspace integration, interface, system, and master release audit test suite | `tests/` | 23 tests |

---

## 2. Additional Project Components

| Component | Path | Responsibility |
|:---|:---|:---|
| **WinUI 3 GUI Dashboard** | `src_gui/CustomWidget.Dashboard` | C# .NET 8 WinUI 3 management dashboard app (12 pages, MVVM architecture, 6 services, 28 tests) |
| **C# SDK Bindings** | `bindings/csharp/CustomWidget.SDK` | C# widget interface (`IWidget`) and MVVM models |
| **TypeScript SDK** | `bindings/typescript/custom-widget-sdk` | TypeScript declarations (`index.d.ts`) |
| **Win32 Hook DLLs** | `native/win32_hooks` | Native C++ WorkerW / Progman desktop shell attachment hooks |
