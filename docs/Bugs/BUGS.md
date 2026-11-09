# Confirmed Bugs

*Verified defects discovered in the Aether codebase.*

---

## BUG-001: `AudioProvider` returns hardcoded static values instead of querying WASAPI

- **Severity**: Medium
- **Component**: `system_providers` — `AudioProvider` in `crates/system_providers/src/providers/mod.rs`
- **Evidence**: The `AudioProvider::sample()` method returns a hardcoded `volume_pct: 75.0` and `is_muted: false` without calling any Win32 API (e.g. `IAudioEndpointVolume`). Other providers like `CpuProvider` and `MemoryProvider` use real Win32 calls. The `WasapiAudioProvider` exists separately for FFT spectrum data but the basic volume/mute metric is still a stub.
- **Impact**: Dashboard and widgets display a static 75% volume regardless of actual system state.
- **Status**: Resolved — implemented COM `IMMDeviceEnumerator` + `IAudioEndpointVolume::GetMasterVolumeLevelScalar` and `GetMute` in `AudioProvider::query_system_audio()`, backed by unit tests.
- **Proposed Fix**: Implement COM-based `IAudioEndpointVolume::GetMasterVolumeLevelScalar` and `GetMute` queries, with a `#[cfg(not(windows))]` fallback.

---

## BUG-002: `ProcessMetricsProvider` uses heuristic estimation rather than actual app classification

- **Severity**: Low
- **Component**: `system_providers` — `ProcessMetricsProvider` in `crates/system_providers/src/providers/mod.rs`
- **Evidence**: The provider calls `EnumProcesses` to get a total process count, then uses arithmetic formulas (`count / 20`, `open_apps * 2`) to estimate `browser_tabs_count`, `gaming_apps_count`, etc. It does not actually classify running processes by window title or executable name.
- **Impact**: Application category counts in telemetry are approximations, not real data.
- **Status**: Resolved — implemented real Win32 process enumeration with `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` and `QueryFullProcessImageNameW` executable classification (`ProcessMetricsProvider::classify_executable`), backed by unit tests.

---

## BUG-003: `TelemetrySnapshot::default()` contains misleading non-zero values

- **Severity**: Low
- **Component**: `system_providers` — `SharedTelemetryCache` in `crates/system_providers/src/shared_cache.rs`
- **Evidence**: `TelemetrySnapshot::default()` initializes `open_apps_count: 5`, `browser_tabs_count: 12`, `battery_charge_pct: 85.0`, `total_gpu_count: 2`, etc. These values are not real defaults — they are hardcoded "realistic-looking" numbers. Before the first telemetry tick, consumers would see data that appears real but is fabricated.
- **Impact**: Brief window of misleading data between engine startup and first telemetry sample.
- **Status**: Resolved — zeroed all pre-tick default fields in `TelemetrySnapshot::default()`, `AudioSpectrumTelemetry::default()`, and `MediaPlaybackTelemetry::default()`, verified by `test_telemetry_snapshot_default_is_zeroed`.

---

## BUG-004: Stale test counts across multiple documentation files

- **Severity**: Medium
- **Component**: Documentation — `AGENTS.md`, `Project_Status.md`, `Detailed_Project_Report.md`, `README.md`, `TESTING.md`
- **Evidence**: `AGENTS.md` states "Current test count: 121". `README.md` badge says "333/333". `Project_Status.md` says "343/343 (313 Rust + 30 C#)". Actual Rust test count from `cargo test --workspace -- --list` is **362**. Multiple docs contradict each other and none match the current reality.
- **Impact**: Contributors cannot trust documented test counts. Governance relies on these numbers.
- **Status**: Resolved — updated across all documentation to 439 tests (380 Rust + 59 C#).

---

## BUG-005: `NamedPipeClient` blocking read loop caused 100% IPC connection timeouts in WinUI 3 Dashboard

- **Severity**: High
- **Component**: `CustomWidget.Dashboard` — `src_gui/CustomWidget.Dashboard/IPCClient/NamedPipeClient.cs`
- **Evidence**: `NamedPipeClient.SendCommandAsync` executed a `while ((bytesRead = await pipeStream.ReadAsync(...)) > 0)` loop without checking for newline or packet completion. Because the Rust daemon stays in its connection loop without closing the pipe, the client's second `ReadAsync` blocked indefinitely until the 500ms timeout fired, throwing `OperationCanceledException` and discarding the valid response.
- **Impact**: The WinUI 3 Dashboard was completely unable to receive telemetry, showing "Disconnected" and flatlined gauges.
- **Status**: Resolved — updated reading logic to break on terminating newline `\n` or single-packet completion, and updated `ipc_server.rs` to write newline-terminated lines.

---

## BUG-006: Missing network throughput in `StatusResponse` caused permanent `0 B/s` display

- **Severity**: Medium
- **Component**: `core_engine` — `StatusResponse` in `crates/core_engine/src/ipc_server.rs` & `EngineStatus.cs`
- **Evidence**: `StatusResponse` in `ipc_server.rs` omitted `net_recv_bytes_per_sec` and `net_sent_bytes_per_sec` despite the fields existing in `TelemetrySnapshot`. Correspondingly, `EngineStatus.cs` and `TelemetrySample.FromStatus` lacked the properties.
- **Impact**: Overview page and Performance page network speedometer and charts displayed `↓ 0 B/s` / `↑ 0 B/s` perpetually.
- **Status**: Resolved — added `net_recv_bytes_per_sec` and `net_sent_bytes_per_sec` to `StatusResponse`, `EngineStatus`, and `TelemetrySample`, backed by automated unit tests.
