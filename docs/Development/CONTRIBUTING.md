# Aether — Developer Contributing Guide

**Developer Onboarding & Local Setup**

---

## 1. Prerequisites & Environment Setup

To build and run the complete Aether stack on Windows 11, install:

1. **Rust Toolchain**: Rust 1.78+ (MSVC toolchain: `x86_64-pc-windows-msvc`).
2. **Visual Studio 2022**: With workload "Desktop development with C++" and "Universal Windows Platform development".
3. **Windows App SDK**: Version 1.5 / 2.2 for WinUI 3 C# Dashboard compilation.
4. **PowerShell 7+**: For running orchestrator launch scripts (`.\launch.ps1`).

---

## 2. One-Command Full Stack Launch

To launch the complete stack (spawns two windows: Engine Daemon and Ratatui TUI Dashboard):

```powershell
.\launch.ps1
```

---

## 3. Step-by-Step Manual Launch

### Terminal 1: Core Engine Daemon
```powershell
cargo run -p core_engine
```

### Terminal 2: Ratatui TUI Dashboard (Start while daemon is running)
```powershell
cargo run -p dashboard_tui
```

### GUI Management Dashboard (WinUI 3 C#)
Open `src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj` in Visual Studio 2022 and press `F5`.

---

## 4. Verification & Testing

Before submitting a Pull Request, run the full automated verification suite:

```powershell
# Run all workspace unit and doc tests
cargo test --workspace

# Confirm zero compilation warnings/errors
cargo check --workspace
```
