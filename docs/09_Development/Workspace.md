# Workspace Layout & Crate Map

**Purpose**: Inventory and responsibilities of all 24 workspace crates in Aether.  
**Audience**: Maintainers, New Developers.  
**Prerequisites**: [Build.md](Build.md).  
**Related Documents**: [Dependency_Graph.md](../01_Architecture/Dependency_Graph.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Workspace Reference  
**Owner**: Core Maintainer Team  

---

## 1. Workspace Crate Map (24 Crates)

| Crate | Responsibility | Test Count |
|---|---|---|
| `core_engine` | Async daemon, IPC pipe server, subsystem orchestrator | 41 tests |
| `system_providers` | Hardware sensor collectors & `SharedTelemetryCache` | 11 tests |
| `widget_sdk` | Standardized 6-pillar widget API & scheduler | 14 tests |
| `ipc_protocol` | IPC types, `ControlCommand`, shared memory ring buffer | 5 tests |
| `recovery_manager` | Crash recovery manager & Safe Mode sentinel | 6 tests |
| `config_manager` | Transactional atomic config & 5-gen backups | 7 tests |
| `capability_broker` | Sandboxing permission broker & `WidgetFirewall` | 7 tests |
| `watchdog` | Heartbeat supervisor daemon | 2 tests |
| `event_recorder` | Time-travel event stream recorder & replayer | 2 tests |
| `observability` | Prometheus exporter, minidump writer & ETW provider | 4 tests |
| `dev_tools` | File-watcher hot-reloader & Chrome DOM inspector | 4 tests |
| `ai_engine` | AI layout synthesizer & wallpaper theme generator | 5 tests |
| `package_manager` | npm-style installer & Ed25519 verifier | 5 tests |
| `enterprise` | Group Policy engine & SHA-256 audit logger | 4 tests |
| `plugin_runtime` | AppContainer sandbox supervisor & integrity guard | 4 tests |
| `theme_engine` | JSON theme parser, hot-swapper & token resolver | 5 tests |
| `animation_engine` | Easing curves & spring physics engine | 3 tests |
| `layout_engine` | Flexbox layout engine | 3 tests |
| `lua_runtime` | Sandboxed Lua 5.4 plugin host | 3 tests |
| `perf_monitor_widget` | Built-in performance card renderer | 4 tests |
| `widget_parser` | TOML widget manifest parser & validator | 2 tests |
| `cloud_sync` | CRDT offline config synchronization | 5 tests |
| `production_engine` | Security auditor, stress harness & auto-updater | 4 tests |
| `dashboard_tui` | Animated Ratatui terminal dashboard | 1 test |

---

## Future Work
- Add `cargo workspace` graph visualization script.

## Known Issues
- None.

## References
- [Cargo.toml](../../Cargo.toml)

## Related Documents
- [Build.md](Build.md)
