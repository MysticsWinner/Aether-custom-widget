# Aether — Testing Architecture & Protocol

**Verification Architecture, Governance Rules, and Test Suite Summary**

---

## 1. Governance Rule: Mandatory Test Enforcement

Per project governance rules:

> **Mandatory Rule**: Regardless of the size or nature of a request, every code change MUST include tests that pass before a task is complete. `cargo test --workspace` must exit with code 0 with zero failing tests.

---

## 2. Test Suite Status Summary

```
Total Workspace Tests: 116 / 116 Passing (100% Pass Rate)
Test Execution Command: cargo test --workspace
Compilation Verification: cargo check --workspace
```

```mermaid
pie title Workspace Test Count by Component (116 Total)
    "core_engine (Unit)" : 41
    "system_providers" : 9
    "widget_sdk" : 8
    "perf_monitor_widget" : 6
    "production_engine" : 6
    "ipc_protocol" : 6
    "package_manager" : 4
    "theme_engine" : 5
    "plugin_runtime" : 5
    "dashboard_tui" : 3
    "lua_runtime" : 2
    "widget_parser" : 2
    "installer" : 1
    "integration_tests" : 8
    "system_tests" : 3
    "interface_tests" : 3
    "other / doc-tests" : 34
```

---

## 3. Test Categories & Scope

| Test Level | Scope | Execution Target | Responsible Framework |
|---|---|---|---|
| **Unit Tests** | Function, method, struct state machine validation | `cargo test --workspace` | Built-in `#[test]` Rust test runner |
| **Doc Tests** | Public API code example validity | `cargo test --doc` | Rustdoc runner |
| **Integration Tests** | IPC named pipe ring buffer & subsystem cross-interaction | `tests/integration_tests.rs` | Async Tokio test runner (`#[tokio::test]`) |
| **System Tests** | E2E lifecycle, chaos recovery, layout persistence | `tests/system_tests.rs` | Async Tokio test runner (`#[tokio::test]`) |
| **Interface Tests** | Serialization compatibility, error handling, health reports | `tests/interface_tests.rs` | Built-in `#[test]` Rust test runner |
| **Micro-Benchmarks** | dirty region tracking, telemetry collect latency | `RainmeterBenchmark` | Benchmark modules within `core_engine` & `widget_sdk` |
