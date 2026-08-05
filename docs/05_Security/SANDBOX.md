# Aether — AppContainer Sandboxing Specification

**Windows AppContainer Isolation and Job Object Constraints**

---

## 1. AppContainer Security Profile

The target security architecture for third-party widget isolation leverages native Windows **AppContainer** sandboxing:

- **Isolated SID**: Widgets run under a dedicated AppContainer Security Identifier (SID) restricting access to host user directories (`C:\Users\...`).
- **Network Isolation**: Outbound network connections are blocked by default unless explicitly granted in `PermissionManifest`.
- **Registry Virtualization**: Direct registry modification is blocked.

---

## 2. Windows Job Object Resource Controls

The `PluginSupervisor` (`crates/plugin_runtime/src/supervisor.rs`) assigns sandboxed widget processes to a Windows **Job Object** enforcing resource quotas:

- **CPU Rate Limiting**: Hard cap on maximum CPU core utilization per widget process.
- **Memory Limit**: Hard limit on maximum commit memory allowance (e.g., 64 MB RAM limit per widget).
- **Process Termination**: If a widget exceeds memory limits, the Job Object terminates the process safely without disrupting the host engine daemon.
