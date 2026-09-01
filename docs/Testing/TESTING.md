# Aether — Testing Architecture & Protocol

**Verification Architecture, Governance Rules, and Test Suite Summary**

---

## 1. Governance Rule: Mandatory Test Enforcement

Per project governance rules:

> **Mandatory Rule**: Regardless of the size or nature of a request, every code change MUST include tests that pass before a task is complete. `cargo test --workspace` and `dotnet test` must exit with code 0 with zero failing tests.

---

## 2. Test Suite Status Summary

```
Total Automated Tests: 333 / 333 Passing (100% Pass Rate)
├── Rust Backend & Integration Suite: 303 Tests
└── C# WinUI 3 GUI Dashboard Suite: 30 Tests
Rust Test Execution Command: cargo test --workspace
C# GUI Test Execution Command: dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj
Compilation Verification: cargo check --workspace && dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj
```

```mermaid
pie title Automated Test Distribution (333 Total Passing Tests)
    "core_engine (74)" : 74
    "widget_sdk (28)" : 28
    "system_providers (22)" : 22
    "theme_engine (13)" : 13
    "config_manager (11)" : 11
    "ai_engine (10)" : 10
    "ipc_protocol (9)" : 9
    "plugin_runtime (9)" : 9
    "capability_broker (7)" : 7
    "production_engine (7)" : 7
    "dev_tools (7)" : 7
    "recovery_manager (6)" : 6
    "observability (6)" : 6
    "perf_monitor_widget (6)" : 6
    "cloud_sync (6)" : 6
    "package_manager (5)" : 5
    "installer (5)" : 5
    "lua_runtime (4)" : 4
    "audio_visualizer_widget (3)" : 3
    "dock_launcher_widget (3)" : 3
    "hardware_pro_widget (2)" : 2
    "weather_particles_widget (2)" : 2
    "crypto_stocks_widget (2)" : 2
    "other crates & widgets (14)" : 14
    "tests/ integration harness (31)" : 31
    "WinUI 3 GUI ViewModel & Telemetry Tests (30)" : 30
```

---

## 3. Test Categories & Scope

| Test Level | Scope | Execution Target | Responsible Framework |
|:---|:---|:---|:---|
| **Unit Tests** | Function, method, struct state machine validation | `cargo test --workspace` | Built-in `#[test]` Rust test runner |
| **Doc Tests** | Public API code example validity | `cargo test --doc` | Rustdoc runner |
| **Integration Tests** | IPC named pipe ring buffer & subsystem cross-interaction | `tests/integration_tests.rs` | Async Tokio test runner (`#[tokio::test]`) |
| **System Tests** | E2E lifecycle, chaos recovery, layout persistence | `tests/system_tests.rs` | Async Tokio test runner (`#[tokio::test]`) |
| **Interface Tests** | Serialization compatibility, error handling, health reports | `tests/interface_tests.rs` | Built-in `#[test]` Rust test runner |
| **Master Release Audit** | Complete 28-crate integration, profile switching, memory resilience | `tests/master_release_audit_tests.rs` | Rust test runner |
| **C# GUI ViewModel Tests** | IPC service, MVVM bindings, profile/theme view models | `dotnet test src_gui/CustomWidget.Dashboard.Tests` | xUnit / MSTest (.NET 8) |
| **Micro-Benchmarks** | dirty region tracking, telemetry collect latency | `RainmeterBenchmark` | Benchmark harness in `benchmarks/` and `core_engine` |
