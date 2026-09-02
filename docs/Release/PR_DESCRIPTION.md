# Aether — Pull Request & Release Description

## 📌 Executive Summary
This release delivers critical reliability bug fixes, architectural decoupling, and cross-tier protocol synchronization across the **WinUI 3 Management Dashboard** (`CustomWidget.Dashboard`) and the **Rust Core Daemon** (`core_engine`, `ipc_protocol`). It establishes robust interface-to-implementation dependency injection, eliminates Named Pipe response truncation for large payloads, ensures UI-thread safety during real-time log ingestion, provides clean synchronous process shutdown coordination, adds backwards-compatible marketplace package search filtering, enables dynamic engine version reporting, and softens working-set garbage collection to eliminate UI micro-stutters.

---

## 🛠️ Detailed Changes by Category

### 1. 🐛 Bug Fixes
- **DI Container Interface-to-Implementation Mapping** (`src_gui/CustomWidget.Dashboard/App.xaml.cs`):
  - **Issue**: Services were registered only by their concrete type (`services.AddSingleton<AetherIpcService>()`), but dependent service and ViewModel constructors requested interface abstractions (`IAetherIpcService`, `IProcessManagerService`, `ITelemetryPollerService`, `ILogCollectorService`, `IMemoryManagerService`, `IWidgetSettingsService`), resulting in `InvalidOperationException` crashes at startup.
  - **Fix**: Implemented dual-binding interface forwarding (`services.AddSingleton<ConcreteService>(); services.AddSingleton<IServiceInterface>(sp => sp.GetRequiredService<ConcreteService>())`). Both concrete and interface injections resolve to the exact same shared singleton instance.
  - **Consumers Updated**: Refactored `MainWindow.xaml.cs`, `DiagnosticsPage.xaml.cs`, and `OverviewPage.xaml.cs` to resolve dependencies via interface contracts.

- **Named Pipe Large-Response Truncation** (`src_gui/CustomWidget.Dashboard/IPCClient/NamedPipeClient.cs`):
  - **Issue**: `SendCommandAsync` performed a single 8 KB `ReadAsync` pass and immediately returned, truncating responses larger than 8 KB (such as `ListWidgets`, `GetDiagnostics`, `GetAuditLogs`, and large marketplace responses) into malformed JSON.
  - **Fix**: Replaced single-read logic with a streaming chunk accumulator loop:
    ```csharp
    int bytesRead;
    while ((bytesRead = await pipeStream.ReadAsync(buffer, linkedCts.Token).ConfigureAwait(false)) > 0)
    {
        ms.Write(buffer, 0, bytesRead);
    }
    ```
    This reads all incoming chunks until EOF upon daemon server connection closure.

- **`ObservableCollection` Cross-Thread Mutation** (`src_gui/CustomWidget.Dashboard/Services/LogCollectorService.cs`):
  - **Issue**: Daemon stdout/stderr lines arriving on background `Task.Run` threads called `Entries.Add()` directly, firing `CollectionChanged` events off the UI thread and causing `RPC_E_WRONG_THREAD` COM exceptions during XAML data binding.
  - **Fix**: Captured `Microsoft.UI.Dispatching.DispatcherQueue` during service initialization and marshaled collection mutations via `_dispatcherQueue.TryEnqueue()`, with thread-safe locking fallback for headless test environments.

- **`async void` Window Closed Delegate Exception Masking** (`src_gui/CustomWidget.Dashboard/MainWindow.xaml.cs`):
  - **Issue**: An `async void` lambda on `Window.Closed` allowed the host application process to exit prematurely before `ShutdownAndCleanAllDependenciesAsync()` completed, leaving orphan engine processes and unobserved exceptions.
  - **Fix**: Switched to a synchronous cleanup handler executing `.GetAwaiter().GetResult()` wrapped in a structured `try-catch` block, ensuring complete engine daemon teardown, working-set release, and log flushing before process termination.

- **SearchMarketplace IPC Protocol Schema Mismatch** (`crates/ipc_protocol/src/messages.rs` & `crates/core_engine/src/ipc_server.rs`):
  - **Issue**: C# dashboard sent `{ "SearchMarketplace": { "query": "...", "category": "all" } }`, while Rust's `ControlCommand::SearchMarketplace` only accepted `{ query: String }`, causing Serde deserialization rejection.
  - **Fix**: Updated the enum variant to include `#[serde(default)] category: Option<String>`, updated pattern matching in `ipc_server.rs`, and verified backwards compatibility for queries omitting the category.

- **Version Reporting Mismatch** (`src_gui/CustomWidget.Dashboard/MainWindow.xaml.cs`):
  - **Issue**: Nav pane footer hardcoded `v0.7.0` while the workspace version in `Cargo.toml` is `0.6.0`.
  - **Fix**: Replaced hardcoded string with dynamic resolution from `ipcService.LastEngineVersion` (sourced from live `GetStatus` responses) with fallback to `0.6.0`.

---

### 2. ⚡ Quality of Life (QoL) & Performance Enhancements
- **Optimized Garbage Collection & Working-Set Trimming** (`src_gui/CustomWidget.Dashboard/Services/MemoryManagerService.cs`):
  - **Adjustment**: Increased auto-cleanup timer interval from 30 seconds to 5 minutes (`TimeSpan.FromMinutes(5)`).
  - **GC Tuning**: Changed GC collection mode from aggressive forced blocking (`GCCollectionMode.Forced, true, true`) to non-blocking optimized collection (`GCCollectionMode.Optimized, false, false`), eliminating UI micro-stutters and frame drops during active telemetry rendering.

---

### 3. 🏛️ Architecture & Design Updates
- **Interface Segregation & Inversion**: All dashboard pages and services now operate against standardized interface contracts (`IAetherIpcService`, `IProcessManagerService`, etc.), improving testability and isolation.
- **Protocol Schema Evolution**: Extended `ControlCommand` with optional fields using serde default attributes, establishing a non-breaking forward/backward compatibility pattern for IPC messages.
- **Updated Architectural Documentation**:
  - `docs/API/CSHARP_API.md`: Updated service architecture and DI pattern reference.
  - `docs/Architecture/IPC_PROTOCOL.md`: Added marketplace search command specification and documented chunked named pipe reading mechanics.

---

### 4. 🧪 Test Suite & Verification Status
- Added Rust unit test verifying serialization roundtrip of `ControlCommand::SearchMarketplace` with and without `category`.
- Added IPC server dispatch test validating response for both `{"SearchMarketplace":{"query":"gpu"}}` and `{"SearchMarketplace":{"query":"gpu","category":"all"}}`.
- All C# unit tests continue to pass against decoupled interface definitions.

---

### 5. 🔒 Security Review
- **AppContainer Sandbox Integrity**: No privilege escalations introduced. IPC Named Pipe continues to operate under strict Windows local-machine ACL permissions (`\\.\pipe\CustomWidgetEngineControlPipe`).
- **Memory Safety**: Buffer allocations in `NamedPipeClient` are bounded by chunk size (`8192` bytes) and automatically disposed via `using MemoryStream`.
- **Exception Isolation**: All shutdown and dispatch paths incorporate logging and fallback boundaries to prevent unhandled domain exceptions.
