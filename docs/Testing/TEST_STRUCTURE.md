# Automated Test Harness & Structure (`tests_suite`)

**Purpose**: Guides workspace unit, integration, interface, and system test execution per AGENTS.md rules.  
**Audience**: All Developers, QA Engineers.  
**Prerequisites**: [Root README](../../README.md).  
**Related Documents**: [BENCHMARK_METHODOLOGY.md](../Performance/BENCHMARK_METHODOLOGY.md), [STRESS_TESTING.md](STRESS_TESTING.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Mandatory Test Protocol  
**Owner**: QA & Testing Lead  

---

## 1. Mandatory Testing Protocol (`AGENTS.md`)

- **Rule**: Every code change MUST include tests verified to pass cleanly before completion.
- **Coverage**: **268 / 268 tests passing** (240 Rust tests + 28 C# GUI tests) across all workspace crates and GUI dashboard.
- **Test Commands**:
  ```powershell
  # Run full Rust workspace test suite:
  cargo test --workspace

  # Run C# WinUI 3 Dashboard unit & service tests:
  dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj

  # Run specific crate tests:
  cargo test -p core_engine
  ```

---

## 2. Test Layer Structure

1. **Unit Tests**: Embedded in `src/lib.rs` and modules of each crate (217 tests).
2. **Integration Tests**: `tests/integration_tests.rs` (8 tests: named pipe IPC, ring buffer, subsystem interactions).
3. **Interface Tests**: `tests/interface_tests.rs` (3 tests: IPC protocol serialization & health checks).
4. **System Tests**: `tests/system_tests.rs` (3 tests: cold restart, chaos injection, e2e lifecycle).
5. **Master Release Audit**: `tests/master_release_audit_tests.rs` (9 tests: full stack multi-profile integration).
6. **GUI ViewModel Tests**: `src_gui/CustomWidget.Dashboard.Tests` (28 tests: IPC client, memory manager, profiles, AI composer, security).

---

## 3. Related Documents
- [TESTING.md](TESTING.md)
- [UNIT_TESTS.md](UNIT_TESTS.md)
- [INTEGRATION_TESTS.md](INTEGRATION_TESTS.md)
- [STRESS_TESTING.md](STRESS_TESTING.md)
- [BENCHMARK_METHODOLOGY.md](../Performance/BENCHMARK_METHODOLOGY.md)
