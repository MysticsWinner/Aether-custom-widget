# Aether — STRIDE Threat Model

**Threat Analysis and Mitigation Strategies**

---

## STRIDE Threat Modeling Matrix

| Threat Category | Potential Risk Scenario | Mitigation Implemented / Planned |
|---|---|---|
| **Spoofing** | Malicious app impersonates engine Named Pipe server. | Named pipe creation uses explicit Security Descriptor (ACL) restricting creation to current user SID. |
| **Tampering** | Untrusted widget modifies shared telemetry cache memory. | `SharedTelemetryCache` exposes read-only snapshot references (`Arc<RwLock>`); direct write access restricted to engine daemon. |
| **Repudiation** | Subsystem state mutation performed without logging. | All IPC control commands and lifecycle events logged to `tracing` subscriber. |
| **Information Disclosure** | Untrusted widget extracts sensitive user data from file system. | AppContainer sandbox blocks file system access outside widget package directory. |
| **Denial of Service** | Malicious widget enters infinite loop consuming 100% CPU. | Windows Job Object enforces CPU rate caps and memory limits; supervisor terminates unresponsive widgets. |
| **Elevation of Privilege** | Malicious widget escapes sandbox to execute arbitrary host code. | AppContainer restricted tokens strip administrator rights and system access privileges. |
