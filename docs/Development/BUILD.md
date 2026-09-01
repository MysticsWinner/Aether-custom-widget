# Build & Compilation Guide

**Instructions for Building, Running, and Packaging Aether Workspace Crates and C# WinUI 3 Dashboard**

---

## 1. System Requirements & Toolchains

| Component | Target Version / Toolchain | Workloads / Features Required |
|:---|:---|:---|
| **Operating System** | Windows 11 (`x86_64` or `ARM64`) | Build 22000 or later |
| **Rust Toolchain** | Rust 1.78+ (`stable-x86_64-pc-windows-msvc`) | Edition 2021 |
| **C# / GUI Toolchain** | .NET 8.0 SDK | Visual Studio 2022 (Windows App SDK 1.5 workload) |
| **C++ Win32 Hooks** | MSVC v143 toolset / CMake 3.20+ | Desktop development with C++ |

---

## 2. Compilation Commands

### 2.1 Workspace Compilation Check
```powershell
# Fast compilation check across all 33 member crates:
cargo check --workspace

# Run clippy strict linter:
cargo clippy --workspace -- -D warnings
```

### 2.2 Building Release Binaries
```powershell
# Build all Rust executables with LTO and optimizations:
cargo build --workspace --release

# Output binaries located in target/release/:
# - core_engine.exe
# - dashboard_tui.exe
# - installer.exe
```

### 2.3 Building C# WinUI 3 Management Dashboard
```powershell
# Restore and compile C# dashboard:
dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj -c Release

# Run C# ViewModel & Service unit tests:
dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj
```

---

## 3. Running the Stack

```powershell
# Quick Start: Launch full stack (Daemon + TUI Dashboard in separate windows):
.\launch.ps1

# Manual Terminal 1: Start background host daemon
cargo run -p core_engine

# Manual Terminal 2: Start Ratatui TUI dashboard
cargo run -p dashboard_tui

# Manual Terminal 3: Start C# WinUI 3 Dashboard (or launch from Visual Studio)
dotnet run --project src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj
```

---

## 4. Local Installer Wizard Generation

To package compiled executables and assets into `%LOCALAPPDATA%\Aether\`:
```powershell
cargo run -p installer
```
The installer packages:
- `core_engine.exe`
- `dashboard_tui.exe`
- `CustomWidget.Dashboard.exe`
- `AetherSetup.exe`
- Built-in widget manifests and theme files into `%LOCALAPPDATA%\Aether\`.
