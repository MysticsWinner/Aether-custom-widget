# Aether — Production Quality Assurance Checklist

**Pre-Flight Verification Protocols for Release Candidate (RC)**

---

## 1. Automated QA Checklist

- [x] **Workspace Unit Tests**: All 268 Rust unit/integration tests pass cleanly via `cargo test --workspace`.
- [x] **Compilation**: `cargo check --workspace` finishes with zero compilation errors.
- [x] **WinUI 3 GUI Build Verification**: `dotnet build src_gui/CustomWidget.Dashboard/CustomWidget.Dashboard.csproj` succeeds cleanly with 0 warnings/errors.
- [x] **WinUI 3 GUI Unit Tests**: All 13 C# MSTest unit tests pass cleanly via `dotnet test src_gui/CustomWidget.Dashboard.Tests/CustomWidget.Dashboard.Tests.csproj`.
- [x] **IPC Named Pipe**: Named Pipe server binds to `\\.\pipe\CustomWidgetEngineControlPipe` without privilege escalation errors.
- [x] **Telemetry Sampling**: CPU percentage and RAM usage update continuously without numerical overflow or panic.

---

## 2. Manual Visual & Functional QA Checklist

### WinUI 3 Management Dashboard (`CustomWidget.Dashboard`)
- [ ] **App Launch**: App launches cleanly from Visual Studio 2022 / executable without unhandled XAML exception.
- [ ] **Visual Theme**: Dark glassmorphism visual styling renders with Mica backdrop and consistent typography.
- [ ] **Overview Page**: 4 live metric gauge cards (CPU, GPU, RAM, NET) display values.
- [ ] **Process Management**: Clicking "Start Engine" launches `core_engine` process and displays live PID.
- [ ] **Diagnostics Console**: Logs stream smoothly without UI freeze; raw IPC commands send/receive correctly.

### Ratatui Terminal Dashboard (`dashboard_tui`)
- [ ] **CLI Launch**: Terminal app connects to live engine Named Pipe.
- [ ] **Gauge Animations**: CPU and RAM gauges update smoothly in terminal without rendering artifacts.
- [ ] **Exit Cleanly**: Pressing `q` exits TUI cleanly restoring terminal state.
