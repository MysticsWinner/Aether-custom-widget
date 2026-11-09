# Aether — IPC Protocol Specification

**Dual-Channel Named Pipe & Shared Memory Ring Buffer IPC Architecture**

---

## 1. Hybrid Dual-Channel Architecture

Aether separates control traffic and high-frequency hardware metrics into two purpose-built channels:

```mermaid
graph LR
    subgraph Host ["Core Engine Host Daemon"]
        PipeServer["Win32 Named Pipe Server"]
        ShmProducer["Shared Memory Ring Buffer Producer"]
    end

    subgraph Clients ["WinUI 3 GUI Dashboard, CLI & AppContainer Plugins"]
        PipeClient["Named Pipe Client"]
        ShmConsumer["Shared Memory Consumer"]
    end

    PipeClient <-->|Control Commands (JSON Text / Async Duplex)| PipeServer
    ShmProducer -->|Zero-Copy Telemetry Stream (MetricPayload)| ShmConsumer
```

### Channel 1: Control Channel (Named Pipes)
- **Pipe Address**: `\\.\pipe\CustomWidgetEngineControlPipe`
- **Transport**: Windows Named Pipe (`tokio::net::windows::named_pipe::ServerOptions`)
- **Protocol**: JSON Line-delimited UTF-8 strings (`\n`)
- **Concurrency**: Asynchronous multi-client listener loop with persistent connection streaming (chunked reads up to 16 KB)
- **Connection Telemetry**: Atomic live tracking of `TOTAL_IPC_CONNECTIONS`, `ACTIVE_IPC_CONNECTIONS`, and `TOTAL_COMMANDS_DISPATCHED`
- **Structured Logging & Diagnostics**:
  - Every connection acceptance, client disconnection, and duration is logged with structured `tracing` fields.
  - Every incoming command and dispatch latency is instrumented at microsecond precision (`dispatch_latency_us`).
  - Corrupted frames or EOF conditions trigger clean resource teardown with structured warnings.
- **Latency**: `< 8 µs` roundtrip

### Channel 2: High-Frequency Telemetry Channel (Shared Memory)
- **Mechanism**: Win32 File Mapping (`CreateFileMappingW`, `MapViewOfFile`)
- **Address**: `Local\AetherTelemetrySharedMemory`
- **Header**: `ShmHeader` (magic: `0x41455448`, protocol_version: `1`, seqlock sequence, timestamp, producer PID, liveness)
- **Payload**: Fixed C-compatible struct `MetricPayload` (`CPU`, `GPU`, `RAM`, `Network`, displays, audio)
- **Multi-Reader Protocols**:
  - `MultiReaderSnapshotBuffer`: Seqlock double-buffered snapshot enabling multiple concurrent readers (WinUI 3 GUI, ratatui TUI, widgets) with zero lock contention and zero reader-reader interference.
  - `MultiReaderRingBuffer`: Circular buffer with monotonic write sequence and independent per-reader cursors (`MultiReaderCursor`).
  - `SharedMemoryRingBuffer`: Dedicated SPSC lock-free circular queue for 1:1 message streaming.
- **Realistic Performance Profile**: Eliminates Win32 Named Pipe IPC serialization and message-copy overhead during steady state. Note that memory-mapped I/O remains subject to OS paging, page faults, CPU cache line invalidations, and Win32 named event synchronization overhead.

---

## 2. Command Variant Reference (`ControlCommand`)

All IPC messages sent over the Control Channel follow Rust Serde's default **externally tagged union** JSON schema:
- **Unit variants** are serialized as simple JSON strings (e.g. `"Ping"`, `"GetStatus"`).
- **Struct variants** are serialized as a single-key JSON object where the key is the variant name and the value is the parameter map (e.g. `{"LoadWidget": {"manifest_path": "..."}}`).

### 2.1 Engine Health, Status & Heartbeat
- **Ping**: `"Ping"` → Response: `{"status": "pong"}`
- **Get Status & Telemetry**: `"GetStatus"` → Response: `{"status":"ok","cpu_pct":...,"gpu_pct":...,"memory_used_mb":...,"memory_total_mb":...,"memory_free_mb":...,"active_widgets":[...],"engine_version":"0.6.0"}`
- **Get Diagnostics**: `"GetDiagnostics"` → Response: `{"pid":...,"uptime_seconds":...,"tick_count":...,"memory_used_mb":...,"total_connections":...,"active_connections":...,"total_commands_dispatched":...}`
- **Get Subsystem Health**: `"GetSubsystemHealth"`
- **Get Health Report**: `"GetHealthReport"`
- **Get Watchdog Status**: `"GetWatchdogStatus"`

### 2.2 Desktop Widget Lifecycle & Overlay
- **Toggle Desktop Widget Overlay**: `"ToggleDesktopWidget"`
- **Reload All Widgets**: `"ReloadAll"`
- **Load Widget**: `{"LoadWidget": {"manifest_path": "path/to/widget.toml"}}`
- **Unload Widget**: `{"UnloadWidget": {"widget_id": "widget_id"}}`
- **Discover Widgets**: `{"DiscoverWidgets": {"search_paths": ["optional/path"]}}` (or `{"search_paths": null}`)
- **List Registered Widgets**: `"ListWidgets"` → Returns array of `WidgetDescriptor` (id, name, state, opacity, scale, locked, enabled, quick_swap, position_x, position_y, manifest_path)
- **Update Display Options**: `{"UpdateWidgetDisplayOptions": {"widget_id": "...", "opacity": 0.85, "scale": 1.0, "locked": false, "enabled": true}}`
- **Quick Swap Widgets**: `{"QuickSwapWidget": {"from_id": "...", "to_id": "...", "mode": "position"}}`
- **Enable Widget**: `{"EnableWidget": {"widget_id": "..."}}`
- **Disable Widget**: `{"DisableWidget": {"widget_id": "..."}}`
- **Set Widget Opacity**: `{"SetWidgetOpacity": {"widget_id": "...", "opacity": 0.8}}`
- **Reset Widget Config**: `{"ResetWidgetConfig": {"widget_id": "..."}}`
- **Set Widget Position**: `{"SetWidgetPosition": {"widget_id": "...", "x": 100, "y": 200}}`
- **Set Widget Lock**: `{"SetWidgetLock": {"widget_id": "...", "locked": true}}`
- **Toggle Widget Lock**: `{"ToggleWidgetLock": {"widget_id": "..."}}`
- **Hot-Reload Widget**: `{"HotReloadWidget": {"widget_id": "..."}}`
- **Inspect Widget**: `{"InspectWidget": {"widget_id": "..."}}`
- **Get Extended Widget Inspector**: `{"GetExtendedWidgetInspector": {"widget_id": "..."}}`
- **Set Widget Render Config**: `{"SetWidgetRenderConfig": {"widget_id": "...", "config_json": "..."}}`
- **Get Widget Render Config**: `{"GetWidgetRenderConfig": {"widget_id": "..."}}`

### 2.3 Recovery, Safe Mode & Quarantine
- **Get Crash History**: `{"GetCrashHistory": {"widget_id": "optional_id"}}`
- **Rollback Widget**: `{"RollbackWidget": {"widget_id": "..."}}`
- **Get Launch Mode**: `"GetLaunchMode"` (Normal vs Safe Mode)
- **Exit Safe Mode**: `"ExitSafeMode"`
- **Get Quarantine List**: `"GetQuarantineList"`
- **Release Quarantine**: `{"ReleaseQuarantine": {"widget_id": "..."}}`

### 2.4 Desktop Snapshots
- **Create Snapshot**: `{"CreateSnapshot": {"name": "Work Setup"}}`
- **List Snapshots**: `"ListSnapshots"`
- **Restore Snapshot**: `{"RestoreSnapshot": {"snapshot_id": "..."}}`
- **Delete Snapshot**: `{"DeleteSnapshot": {"snapshot_id": "..."}}`
- **Export Snapshot**: `{"ExportSnapshot": {"snapshot_id": "...", "path": "backup.json"}}`
- **Import Snapshot**: `{"ImportSnapshot": {"path": "backup.json"}}`

### 2.5 Security, Capability Broker & Enterprise
- **Request Capability Token**: `{"RequestCapabilityToken": {"widget_id": "...", "capability": "NetworkAccess"}}`
- **Revoke Capability Token**: `{"RevokeCapabilityToken": {"token_id": "..."}}`
- **Get Widget Resource Usage**: `{"GetWidgetResourceUsage": {"widget_id": "..."}}`
- **Get Capability Grants**: `{"GetCapabilityGrants": {"widget_id": null}}`
- **Get Enterprise Policy**: `"GetEnterprisePolicy"`
- **Update Enterprise Policy**: `{"UpdateEnterprisePolicy": {"policy_json": "..."}}`
- **Get Audit Logs**: `"GetAuditLogs"`
- **Verify Audit Chain**: `"VerifyAuditChain"`

### 2.6 Event Recording & Time-Travel Debugging
- **Start Recording**: `"StartRecording"`
- **Stop Recording**: `"StopRecording"`
- **Get Recording**: `"GetRecording"`
- **Replay Recording**: `{"ReplayRecording": {"from_seq": 0}}`

### 2.7 Observability & Chaos Testing
- **Inject Chaos Failure**: `{"InjectChaosFailure": {"scenario": "GpuDeviceLost"}}`
- **Get Observability Status**: `"GetObservabilityStatus"`
- **Get Prometheus Metrics**: `"GetPrometheusMetrics"`
- **Generate Minidump**: `{"GenerateMinidump": {"reason": "ManualDiagnostics"}}`
- **List Minidumps**: `"ListMinidumps"`
- **Get Scheduler Status**: `"GetSchedulerStatus"`
- **Set Adaptive Tick Mode**: `{"SetAdaptiveTickMode": {"enabled": true}}`
- **Get Resource Cache Stats**: `"GetResourceCacheStats"`
- **Toggle Layout Grid**: `{"ToggleLayoutGrid": {"enabled": true}}`

### 2.8 AI Desktop Composer, Themes & Marketplace
- **Synthesize Desktop**: `{"SynthesizeDesktop": {"prompt": "Triple 4K Dev Workstation"}}`
- **Synthesize Widget**: `{"SynthesizeWidget": {"prompt": "Stock ticker for AAPL"}}`
- **Generate Wallpaper Theme**: `{"GenerateWallpaperTheme": {"wallpaper_path": null}}`
- **Get AI Performance Advice**: `{"GetAiPerformanceAdvice": {"widget_id": null}}`
- **Search Marketplace**: `{"SearchMarketplace": {"query": "monitoring", "category": null}}`
- **Set Theme Mode**: `{"SetThemeMode": {"mode": "dark"}}`
- **Resolve Design Tokens**: `{"ResolveDesignTokens": {"theme_id": null}}`
- **Set Accessibility Mode**: `{"SetAccessibilityMode": {"high_contrast": null, "reduce_motion": null, "reduce_transparency": null, "large_text": null}}`
- **Set Desktop Profile**: `{"SetDesktopProfile": {"profile_id": "Coding"}}`

### 2.9 Garbage Collection & Runtime Policy
- **Trigger Garbage Collection**: `{"TriggerGarbageCollection": {"aggressive": true, "trim_working_set": true}}` → Returns `GarbageCollectionReport` (`collections_count`, `last_reclaimed_kb`, `dev_mode`, `aggressive`, `gc_interval_secs`, `resource_cache_items`, `resource_cache_capacity`, `memory_used_mb`)
- **Set Garbage Collection Policy**: `{"SetGarbageCollectionPolicy": {"dev_mode": true, "aggressive": true, "interval_secs": 15}}`
- **Get Garbage Collection Stats**: `"GetGarbageCollectionStats"` → Returns `GarbageCollectionReport`

- **Get Active Profile**: `"GetActiveProfile"`
- **List Profiles**: `"ListProfiles"`

---

## 3. Client Implementation Matrix

| Client | Implementation Path | Language | Notes |
|:---|:---|:---|:---|
| **WinUI 3 Dashboard** | `src_gui/CustomWidget.Dashboard/Services/AetherIpcService.cs` | C# (.NET 8) | Async `NamedPipeClientStream` with chunked memory-stream reading (`while > 0`), timeout, exponential backoff retries, power/visibility-aware adaptive polling (500ms on AC, 2000ms on battery, 5000ms minimized), and thread-safe volatile connection tracking. |
| **Ratatui TUI** | `crates/dashboard_tui/src/main.rs` | Rust | Tokio `ClientOptions` pipe client with live polling loop. |
| **CLI Tools** | `crates/dev_tools/src/ipc.rs` | Rust | Low-latency synchronous and async pipe helpers. |

