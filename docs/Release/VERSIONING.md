# Aether — Semantic Versioning Policy

**SemVer Rules and API Compatibility Controls**

---

## 1. Semantic Versioning Rules (`MAJOR.MINOR.PATCH`)

Aether enforces strict [SemVer 2.0.0](https://semver.org/) rules across all workspace crates and SDK APIs:

- **MAJOR**: Breaking changes to IPC JSON protocol, `WidgetLifecycle` trait signature, or manifest schemas.
- **MINOR**: Backward-compatible addition of new IPC commands, SDK methods, or subsystems.
- **PATCH**: Backward-compatible bug fixes and internal performance optimizations.

---

## 2. API Version Checking (`plugin_runtime::compatibility`)

Widget plugins specify their API version inside `widget.toml`:

```toml
[metadata]
id = "my_widget"
api_version = "1.2.0"
```

The `CompatibilityChecker` verifies major version equivalence with `HOST_API_VERSION` (`1.x.x`). Plugins requiring major version mismatches are prevented from loading.
