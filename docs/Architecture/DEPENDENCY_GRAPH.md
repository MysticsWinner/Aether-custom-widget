# Dependency Graph & Workspace Topology

**Purpose**: Maps crate dependency graph and prevents circular dependencies across Aether workspace crates.  
**Audience**: Maintainers, Build Engineers.  
**Prerequisites**: [System_Architecture.md](System_Architecture.md).  
**Related Documents**: [Workspace.md](../09_Development/Workspace.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Reference  
**Owner**: Core Architecture Team  

---

## 1. Crate Dependency Graph

```
                   ┌──────────────────┐
                   │   core_engine    │
                   └────────┬─────────┘
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  widget_sdk  │    │ ipc_protocol │    │dev_tools, etc│
└───────┬──────┘    └──────────────┘    └──────────────┘
        ▼
┌──────────────────┐
│system_providers  │
└──────────────────┘
```

> **Circular Dependency Guard Rule**: `ipc_protocol` MUST NOT depend on `widget_sdk`. IPC commands passing complex configs use stringified JSON (`config_json: String`) to preserve clear acyclic dependency trees.

---

## Future Work
- Enforce crate dependency graph constraints via `cargo-deny` in CI.

## Known Issues
- None.

## References
- [Cargo.toml](../../Cargo.toml)

## Related Documents
- [Workspace.md](../09_Development/Workspace.md)
