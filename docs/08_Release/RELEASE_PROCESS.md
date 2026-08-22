# Aether — Production Release Process

**Build, Package, Sign, and Distribution Pipeline**

---

## 1. Release Build Steps

To prepare a production release candidate:

```powershell
# 1. Run workspace test verification
cargo test --workspace

# 2. Compile release binaries with LTO optimizations
cargo build --workspace --release

# 3. Compile WinUI 3 C# Dashboard in Release mode
dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj -c Release
```

---

## 2. Packaging & Installer Creation (`crates/installer`)

The `installer` crate generates the local application directory distribution containing only compiled binaries and assets (no source code):

```
%LOCALAPPDATA%\Aether\
├── bin/
│   ├── core_engine.exe
│   ├── dashboard_tui.exe
│   ├── CustomWidget.Dashboard.exe
│   └── AetherSetup.exe
└── data/
    ├── themes/
    └── widgets/
```

Execution command:
```powershell
cargo run -p installer -- --install
```

---

## 3. Git Push / Pull Governance & PR Release Documentation

1. **Explicit Request Policy**:
   - `git push` and `git pull` are NEVER executed automatically after every commit/prompt.
   - Only execute `git push` / `git pull` when explicitly instructed by the user.

2. **Detailed PR / Commit Breakdown (`docs/08_Release/PR_DESCRIPTION.md`)**:
   - Whenever an explicit push/pull or release is requested, update `PR_DESCRIPTION.md` comparing current changes against the previous commit/PR.
   - Outline:
     - **New Features**: Newly added APIs, UI components, background engines, or tools.
     - **Bug Fixes**: Regression fixes and edge-case resolution.
     - **Quality of Life (QoL)**: Developer experience improvements, UI polish, workflow automation.
     - **Architecture & Design**: Subsystem decouplings, interface isolation, state mutability refinements.
     - **Performance & Benchmarks**: Latency metrics, RAM usage, dirty-rectangle rendering efficiency.
     - **Automated Test Counts**: Exact number of passing Rust workspace tests + C# WinUI 3 tests compared to previous release.

3. **Post-Push Local Installer Creation**:
   - Immediately following an explicit push/pull or release command, build and run the local installer wizard:
     ```powershell
     cargo run -p installer -- --install
     ```
   - Verifies all binaries (`core_engine.exe`, `dashboard_tui.exe`, `CustomWidget.Dashboard.exe`, `AetherSetup.exe`) are deployed cleanly to `%LOCALAPPDATA%\Aether\bin` without any source code files (`.rs`, `.cs`, `.toml` code).

