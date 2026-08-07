# Build & Compilation Guide

**Purpose**: Instructions for compiling the Aether workspace crates and C# WinUI 3 dashboard.  
**Audience**: All Developers, New Contributors.  
**Prerequisites**: Rust 1.78+, Visual Studio 2022 / Windows App SDK 1.5.  
**Related Documents**: [Workspace.md](Workspace.md), [Contributing.md](Contributing.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Build Instructions  
**Owner**: DevOps & Build Team  

---

## 1. System Requirements

- **OS**: Windows 11 (`x86_64` or `ARM64`).
- **Rust Toolchain**: `stable-x86_64-pc-windows-msvc` (Rust 2021 edition).
- **C# Toolchain**: .NET 8 SDK + Visual Studio 2022 (with "Desktop development with C++" and "Universal Windows Platform development" workloads).

---

## 2. Build Commands

```powershell
# Verify workspace compilation:
cargo check --workspace

# Build release binaries:
cargo build --workspace --release

# Run one-command launcher (Daemon + TUI Dashboard):
.\launch.ps1
```

---

## Future Work
- Add automated NSIS setup installer build script.

## Known Issues
- None.

## References
- [Cargo.toml](../../Cargo.toml)

## Related Documents
- [Workspace.md](Workspace.md)
- [Contributing.md](Contributing.md)
