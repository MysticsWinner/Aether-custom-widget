# Aether — Security Architecture Overview

**Security Posture, Isolation Boundaries, and Least Privilege**

---

## 1. Security Architecture Principles

Aether enforces defense-in-depth principles across all platform layers:

- **Least Privilege Execution**: Engine core daemon runs as a standard user process; root/administrator privileges are strictly avoided unless required for custom hardware drivers.
- **Process Isolation**: GUI management dashboard runs in a separate process space from the core engine daemon.
- **Sandboxed Widgets**: Third-party widget plugins execute within restricted container boundaries managed by `PluginSupervisor`.
- **Signed Package Verification**: Widget packages are verified via cryptographic signatures before installation.

---

## 2. Threat & Protection Boundaries

```mermaid
graph TD
    subgraph Host ["Host OS Environment (Windows 11)"]
        Daemon["Aether Engine Daemon (Standard User)"]
        Pipe["Named Pipe (\\\\.\\pipe\\CustomWidgetEngineControlPipe)"]
        
        subgraph Sandbox ["AppContainer Sandbox"]
            Widget1["Third-Party Widget Plugin A"]
            Widget2["Third-Party Widget Plugin B"]
        end
    end

    Pipe <--> Daemon
    Daemon -->|Restricted Token & Job Object| Sandbox
    Sandbox -.x|Blocked Host File Access| Host
```
