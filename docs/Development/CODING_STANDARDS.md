# Aether — Coding Standards & Guidelines

**Mandatory Engineering Rules and Conventions**

---

## 1. Governance & Workflow Rules

As defined in the project's development guidelines:

1. **Phased Architecture Roadmap**: Never skip phases or implement features whose underlying architecture has not been approved.
2. **Interface Isolation**: Every subsystem must expose abstract interfaces (`Subsystem` trait, `GpuRenderer` trait, `IWidget` interface) instead of concrete types.
3. **Composition Over Inheritance**: Prefer component composition over deep inheritance hierarchies.
4. **State & Immutability**:
   - Avoid global mutable state.
   - Prefer thread-safe, immutable data structures (`Arc<RwLock<T>>`) and pure zero-side-effect functions.
5. **Dependency Management**: Audit external dependencies regularly; keep crate dependencies minimal.
6. **Cross-Architecture Support**: Everything must compile cleanly on Windows 11 for both `x86_64` and `ARM64`.

---

## 2. Mandatory Testing Protocol

> **Rule**: Every code change MUST include tests that pass before the task is considered complete.

### Minimum Test Requirements by Change Type

| Change Type | Minimum Requirement |
|---|---|
| New public function / method | $\ge 1$ unit test per function |
| New struct / trait impl | $\ge 1$ lifecycle / happy-path test |
| New crate | $\ge 3$ tests: normal path, edge case, error case |
| Bug fix | $\ge 1$ regression test proving bug is fixed |
| Refactor | All pre-existing workspace tests must pass |
| Manifest / config schema | $\ge 1$ parse / roundtrip test |

### Test Naming Convention
```rust
#[test]
fn test_<unit_under_test>_<scenario>() { ... }

// Examples:
#[test]
fn test_cpu_provider_percentage_in_range() { ... }

#[test]
fn test_perf_widget_lifecycle_mounts_cleanly() { ... }
```

---

## 3. Rust Code Standards

- **Formatting**: Format code using standard `cargo fmt`.
- **Linting**: Ensure `cargo check --workspace` and `cargo clippy --workspace` exit with zero warnings.
- **Unsafe Code**: Forbidden unless required for native Win32 FFI bindings. All `unsafe` blocks must feature a `// SAFETY:` comment explaining memory safety invariants.

---

## 4. C# / WinUI 3 Standards

- **MVVM Pattern**: Use `CommunityToolkit.Mvvm` attributes (`[ObservableProperty]`, `[RelayCommand]`) for view models.
- **Async Naming**: Suffix all asynchronous C# methods with `Async` (e.g. `SendControlCommandAsync`).
- **UI Thread Dispatching**: Ensure telemetry callbacks updating UI components use `DispatcherQueue.TryEnqueue()`.
