# Aether — Code Review Guidelines & Checklist

**Pull Request Criteria and Verification Protocols**

---

## 1. Pre-Submission Review Checklist

Before marking any Pull Request ready for review, developers must confirm compliance with the following:

- [ ] **Compilation**: `cargo check --workspace` passes cleanly on Windows 11 MSVC without warnings.
- [ ] **Unit Tests**: `cargo test --workspace` passes 100% of tests (**87/87 minimum**).
- [ ] **New Tests**: Every new public function, struct, or bug fix includes corresponding unit tests following naming conventions (`test_<unit>_<scenario>`).
- [ ] **Documentation**: Any updated interfaces, structs, or API endpoints are documented with doc comments (`///`).
- [ ] **No Unsafe Code Without Justification**: New `unsafe` blocks include explicit `// SAFETY:` rationale comments.
- [ ] **No Uncaught Panics**: Async tasks handle errors via `Result` rather than `.unwrap()` or `.expect()`.
- [ ] **WinUI 3 GUI Verification**: Verified that WinUI 3 dashboard builds and launches cleanly in Visual Studio 2022.

---

## 2. Mandatory Architectural Review Checklist

1. **"Collect Once, Publish Everywhere"**: Verify no new direct OS queries are introduced inside widgets or IPC handlers. All hardware telemetry must be read from `SharedTelemetryCache`.
2. **Lock Safety**: Verify `RwLock` read/write guard scopes are strictly minimized and never held across await points.
3. **IPC Wire Protocol**: Confirm any changes to `ControlCommand` or `MetricPayload` maintain serde backward compatibility.
