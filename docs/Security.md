# Aether Security & Sandboxing Specification

## Overview

**Aether Platform** implements a zero-trust security architecture ensuring that 3rd-party widget plugins cannot compromise system integrity or crash the host daemon.

---

## Security Layers

### 1. Windows AppContainer Sandboxes
- Plugins run under Low-Integrity SIDs (`S-1-15-2...`).
- Blocked from writing to registry hives or user document directories.

### 2. JobObject Resource Limits
- **CPU Quota**: Hard cap at 2.0% per plugin process.
- **Memory Limit**: 50 MB working set cap. Exceeding limits triggers automated sandbox termination and restart.

### 3. Ed25519 Cryptographic Verification
- All `.cwp` package archives installed via **Aether CLI** must pass Ed25519 digital signature validation before extraction.

### 4. PermissionGuard Capability Checks
- Widgets must declare requested permissions in `widget.toml` (e.g., `capability.network.http`). Unrequested calls are blocked at the IPC gateway.
