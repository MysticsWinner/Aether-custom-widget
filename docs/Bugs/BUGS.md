# Confirmed Bugs

*Verified defects discovered in the Aether codebase.*

---

## BUG-001: `AudioProvider` returns hardcoded static values instead of querying WASAPI

- **Severity**: Medium
- **Component**: `system_providers` — `AudioProvider` in `crates/system_providers/src/providers/mod.rs`
- **Evidence**: The `AudioProvider::sample()` method returns a hardcoded `volume_pct: 75.0` and `is_muted: false` without calling any Win32 API (e.g. `IAudioEndpointVolume`). Other providers like `CpuProvider` and `MemoryProvider` use real Win32 calls. The `WasapiAudioProvider` exists separately for FFT spectrum data but the basic volume/mute metric is still a stub.
- **Impact**: Dashboard and widgets display a static 75% volume regardless of actual system state.
- **Status**: Open
- **Proposed Fix**: Implement COM-based `IAudioEndpointVolume::GetMasterVolumeLevelScalar` and `GetMute` queries, with a `#[cfg(not(windows))]` fallback.

---

## BUG-002: `ProcessMetricsProvider` uses heuristic estimation rather than actual app classification

- **Severity**: Low
- **Component**: `system_providers` — `ProcessMetricsProvider` in `crates/system_providers/src/providers/mod.rs`
- **Evidence**: The provider calls `EnumProcesses` to get a total process count, then uses arithmetic formulas (`count / 20`, `open_apps * 2`) to estimate `browser_tabs_count`, `gaming_apps_count`, etc. It does not actually classify running processes by window title or executable name.
- **Impact**: Application category counts in telemetry are approximations, not real data.
- **Status**: Open (documented in existing tech debt as TD-03 variant)

---

## BUG-003: `TelemetrySnapshot::default()` contains misleading non-zero values

- **Severity**: Low
- **Component**: `system_providers` — `SharedTelemetryCache` in `crates/system_providers/src/shared_cache.rs`
- **Evidence**: `TelemetrySnapshot::default()` initializes `open_apps_count: 5`, `browser_tabs_count: 12`, `battery_charge_pct: 85.0`, `total_gpu_count: 2`, etc. These values are not real defaults — they are hardcoded "realistic-looking" numbers. Before the first telemetry tick, consumers would see data that appears real but is fabricated.
- **Impact**: Brief window of misleading data between engine startup and first telemetry sample.
- **Status**: Open

---

## BUG-004: Stale test counts across multiple documentation files

- **Severity**: Medium
- **Component**: Documentation — `AGENTS.md`, `Project_Status.md`, `Detailed_Project_Report.md`, `README.md`, `TESTING.md`
- **Evidence**: `AGENTS.md` states "Current test count: 121". `README.md` badge says "333/333". `Project_Status.md` says "343/343 (313 Rust + 30 C#)". Actual Rust test count from `cargo test --workspace -- --list` is **362**. Multiple docs contradict each other and none match the current reality.
- **Impact**: Contributors cannot trust documented test counts. Governance relies on these numbers.
- **Status**: Open — being fixed in this documentation overhaul.
