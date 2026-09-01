# AppContainer Sandboxing & Process Isolation

**AppContainer SID Configuration, JobObject Limits, and IPC Isolation**

---

## 1. AppContainer Sandboxing (`plugin_runtime`)

Aether executes 3rd-party widget plugins and Lua runners in hardware-isolated Windows **AppContainer** sandboxes under low-integrity SIDs (`S-1-15-2-...`).

### 1.1 Process Lifecycle Management
- **Supervisor**: `PluginSupervisor` in `crates/plugin_runtime` manages sandbox process creation via `CreateProcessAsUserW` or `CreateAppContainerProfile`.
- **Integrity Guard**: Checks memory integrity and prevents unauthorized DLL injection.
- **Crash Recovery**: Automatically terminates and isolates misbehaving plugins; restarts up to 3 times before quarantining.

---

## 2. JobObject Limits & Enforcement

Every sandbox process is placed inside a dedicated Windows `JobObject`:
- **CPU Quota**: Max 2.0% CPU time.
- **RAM Working Set**: Max 50 MB RAM allocation limit.
- **Child Processes**: Process creation blocked (`JOB_OBJECT_LIMIT_ACTIVE_PROCESS = 1`).
