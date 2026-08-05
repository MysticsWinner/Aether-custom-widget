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

The `installer` crate generates the local application directory distribution:

```
%LOCALAPPDATA%\Aether\
├── bin/
│   ├── core_engine.exe
│   ├── dashboard_tui.exe
│   └── CustomWidget.Dashboard.exe
└── data/
    ├── themes/
    └── widgets/
```

Execution command:
```powershell
cargo run -p installer -- --install
```
