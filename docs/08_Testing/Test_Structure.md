# Automated Test Harness & Structure (`tests_suite`)

**Purpose**: Guides workspace unit, integration, interface, and system test execution per AGENTS.md rules.  
**Audience**: All Developers, QA Engineers.  
**Prerequisites**: [Root README](../../README.md).  
**Related Documents**: [Benchmarks.md](Benchmarks.md), [Stress_Testing.md](Stress_Testing.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Mandatory Test Protocol  
**Owner**: QA & Testing Lead  

---

## 1. Mandatory Testing Protocol (`AGENTS.md`)

- **Rule**: Every code change MUST include tests verified to pass cleanly before completion.
- **Coverage**: **184 / 184 tests passing** across 24 workspace crates.
- **Test Commands**:
  ```powershell
  # Run full workspace test suite:
  cargo test --workspace

  # Run specific crate tests:
  cargo test -p core_engine
  ```

---

## 2. Test Layer Structure

1. **Unit Tests**: Embedded in `src/lib.rs` and modules of each crate.
2. **Integration Tests**: `tests/integration_tests.rs` (subsystem interaction).
3. **Interface Tests**: `tests/interface_tests.rs` (IPC protocol serialization & health checks).
4. **System Tests**: `tests/system_tests.rs` (cold restart, chaos injection, e2e lifecycle).

---

## Future Work
- Add code coverage calculation script via `cargo-tarpaulin`.

## Known Issues
- None.

## References
- [tests/integration_tests.rs](file:///d:/Code/Aether-custom-widget/tests/integration_tests.rs)
- [AGENTS.md](../../.agents/AGENTS.md)

## Related Documents
- [Benchmarks.md](Benchmarks.md)
