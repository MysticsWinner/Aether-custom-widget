# Aether v0.8 — Gap Analysis & Requirements Plan

Nov 24, 2026 · @Tanmay

## Summary

The repo is now at `19f2e6f` (v0.7.0, 4 commits pulled from origin/main). Every doc marks all 26 phases and 33 crates "Completed", and the test count is real (405 Rust + 63 C# tests). But at least 9 headline features are simulated or missing in code, so the next release should be about making claims true, not adding new ones.

- **Most serious:** the IPC named pipe has no access control or caller check, so any local process can send any of the 60+ commands.
- **Most misleading:** "AES-256-GCM" cloud sync, the "wasmtime" WASM runtime and "ONNX/Whisper" AI have no crypto, WASM or ONNX dependency at all.
- **Most visible to users:** per-core CPU, the audio spectrum, now-playing media, weather and crypto prices show generated or fixed values.

This plan lists 14 gaps, 14 functional and 12 non-functional requirements (26 total), and a 4-milestone path to v0.8.

Note on the pull: the mounted copy showed 378 "modified" files. These were only file-mode and line-ending differences (zero content changes), so the pull ran with those ignored. Delete permission was enabled for this folder so git could replace files.

## What Aether is today

Aether is a Windows 11 desktop-widget engine (a Rainmeter alternative): a Rust daemon renders widgets behind desktop icons and serves telemetry to a WinUI 3 dashboard over a named pipe.

| Layer | What exists | Size / evidence |
| --- | --- | --- |
| Core daemon | `core_engine`: Tokio 10 ms tick loop, subsystem manager, event bus, IPC server, WorkerW + DirectComposition + D2D rendering | 1,555-line `ipc_server.rs`; real `D3D11CreateDevice`, `D2D1CreateFactory` |
| Telemetry | `system_providers`: CPU, RAM, GPU (D3DKMT), network (`GetIfTable2`), audio volume, processes, shared atomic cache | 1,012-line `providers/mod.rs` |
| Widget runtimes | `widget_sdk`, `widget_parser` (TOML), `lua_runtime` (mlua 5.4), `plugin_runtime` (AppContainer + "WASM") | 21 SDK files |
| Platform services | theme, layout (taffy), animation, package manager (ed25519), cloud sync, AI engine, recovery, config transactions, capability broker, watchdog, observability, enterprise GPO, installer | 33 crates total |
| Showcase widgets | perf monitor, weather, weather particles, network, audio visualizer, hardware pro, dock launcher, crypto/stocks, AI assistant | 9 crates |
| GUIs | WinUI 3 dashboard (13 pages, MVVM) and a Ratatui TUI | `src_gui/`, `dashboard_tui` |
| SDKs | C# and TypeScript bindings, C++ `win32_hooks` | `bindings/`, `native/` |
| Quality | 405 Rust + 63 C# tests; CI on windows-latest for x64 + ARM64 | `.github/workflows/ci.yml` |
| Docs | ~90 Markdown files across 15 folders, ADR-001 to 012, bug/debt registers | `docs/` |

All entries in `docs/Bugs/` (6 bugs, 7 debt items, 3 architecture concerns) are already marked Resolved. The gaps below were found by reading the code, not the registers.

## What is missing

14 gaps, sorted by severity. Two are security holes that should block any release.

| # | Gap | Evidence | Severity |
| --- | --- | --- | --- |
| G1 | IPC pipe has no security descriptor and no caller check. Any process in the session can load widgets, change policy or mint capability tokens. | `core_engine/src/ipc_server.rs:248` uses `ServerOptions::new()` with defaults; no `GetNamedPipeClientProcessId` or auth handshake | Critical |
| G2 | Lua widgets run with the full safe stdlib, so `io` and `os` (file access, `os.execute`) are available to any widget script. | `lua_runtime/src/lib.rs:29` calls `Lua::new()`; no memory or instruction limit | Critical |
| G3 | Cloud sync claims AES-256-GCM but has no crypto dependency; data is not encrypted. | `cloud_sync/src/manager.rs:7` doc comment; `Cargo.toml` has only serde/tokio | High |
| G4 | WASM runtime only checks the header; `invoke_with_fuel` returns computed numbers and never executes code. | `plugin_runtime/src/wasm.rs:125-157`; no `wasmtime` dependency | High |
| G5 | Auto-updater calls `reqwest::blocking` from inside the Tokio daemon, which panics in an async context. | `production_engine/src/auto_updater.rs:70` | High |
| G6 | "ONNX / Whisper" AI is hand-written softmax and RMS voice detection; no model is loaded. | `ai_engine/src/onnx_runtime.rs`; no `ort` or whisper dependency | Medium |
| G7 | Per-core CPU load is generated with `sin()` around the overall average. | `system_providers/src/providers/cpu_topology.rs:140` | Medium |
| G8 | Audio spectrum is a sine animation scaled by volume; "now playing" is a hardcoded track ("Resonance (Synthwave Mix)"). | `providers/wasapi_audio.rs:101, 131` | Medium |
| G9 | Weather widget starts at a fixed 22.5 °C / 55 % with no data source; crypto prices are constants (BTC $64,250). | `weather_widget/src/lib.rs:30`, `providers/crypto_financial.rs:31` | Medium |
| G10 | Benchmark claims (0.08 % CPU, 22 MB, 45 ms vs Rainmeter) are static Markdown with no script that reproduces them. | `benchmarks/*.md` | Medium |
| G11 | CI is soft: `cargo fmt` is `continue-on-error`, clippy is installed but never run, no `cargo audit`/`deny`, no coverage. | `.github/workflows/ci.yml:26-30` | Medium |
| G12 | Dashboard has no localization and no accessibility markup. | 0 `.resw` files; 0 of 13 XAML pages use `AutomationProperties` | Medium |
| G13 | C++ `win32_hooks` is never built or linked: no `build.rs` and no crate references it. | `native/win32_hooks/` | Low |
| G14 | Repo hygiene: README links a missing `LICENSE`; no `SECURITY.md`; README shows 333 and 268 tests vs 468 actual; no `.gitattributes`, so Windows checkouts show hundreds of mode/EOL-only changes. | `README.md`, repo root | Low |

## Functional requirements

14 requirements. Each closes a gap (G#) or fills a missing user-facing capability, and each has a test that proves it.

| ID | Requirement | Acceptance criteria | Closes | Priority |
| --- | --- | --- | --- | --- |
| FR-01 | Authenticated IPC | Pipe created with an SDDL allowing only the current user SID; server checks client PID/image path; commands are split into read-only and privileged, and privileged ones need a session token issued at dashboard launch. | G1 | P0 |
| FR-02 | Sandboxed Lua | Lua state built with only `base`, `string`, `table`, `math`, `utf8`; `io`, `os`, `package`, `debug` absent; memory cap (default 16 MB) and instruction hook abort runaway scripts. | G2 | P0 |
| FR-03 | Real sync encryption | Payloads encrypted with AES-256-GCM (`aes-gcm` crate), key from Windows DPAPI, unique 96-bit nonce per blob; tamper test fails decryption. | G3 | P0 |
| FR-04 | Real WASM execution | Modules run in `wasmtime` with fuel metering, memory limits and only the capability-gated host imports; a sample widget compiled from Rust runs end to end. | G4 | P1 |
| FR-05 | Safe auto-update | Async HTTP client, manifest and package verified with the existing ed25519 key before install, rollback via `recovery_manager` on failed start. | G5 | P0 |
| FR-06 | Real per-core CPU | Per-core load from `NtQuerySystemInformation(SystemProcessorPerformanceInformation)` or PDH; values match Task Manager within ±3 points. | G7 | P1 |
| FR-07 | Real audio spectrum | WASAPI loopback capture into a 1024-point FFT (`rustfft`), 16 log bins, at most 60 Hz. | G8 | P1 |
| FR-08 | Real now-playing | Title, artist, position and status from WinRT `GlobalSystemMediaTransportControlsSessionManager`; empty state when nothing plays. | G8 | P1 |
| FR-09 | Live weather and market data | Pluggable data-source trait with one keyless weather provider (e.g. Open-Meteo) and one market provider; user-set location and symbols; caching; requests go through the capability broker's network permission. | G9 | P1 |
| FR-10 | Honest AI features | Either load a real ONNX model via `ort` behind a feature flag, or rename to "rule-based intent parser" in code and docs. | G6 | P2 |
| FR-11 | Native hook build | `build.rs` builds `win32_hooks` with `cc`/CMake and links it, or the folder is removed and docs updated. | G13 | P2 |
| FR-12 | Widget authoring in the dashboard | Create, edit and hot-reload a `widget.toml` + Lua widget from the dashboard with live validation errors shown inline. | New | P2 |
| FR-13 | Layout backup and restore | Export and import full desktop layout (widgets, positions, themes, profiles) as one signed file; restore works on a clean machine. | New | P2 |
| FR-14 | Settings and onboarding | First-run wizard (start with Windows, default widgets, telemetry opt-in); every setting persisted and reflected in the dashboard. | New | P2 |

## Non-functional requirements

12 requirements, each with a number CI or a reviewer can check.

| ID | Area | Requirement and target | Closes | Priority |
| --- | --- | --- | --- | --- |
| NFR-01 | Truthful telemetry | No synthetic values on any path compiled for Windows. A CI grep fails on `sin(`/hardcoded fixtures in `providers/` outside `#[cfg(test)]`; providers that can't read data report `available: false`. | G7–G9 | P0 |
| NFR-02 | Security review | STRIDE threat model updated for IPC, Lua, WASM, updater and sync; fuzz the IPC JSON parser and `widget.toml` parser with `cargo-fuzz` for 1 h per release, zero crashes. | G1–G5 | P0 |
| NFR-03 | Supply chain | `cargo audit` and `cargo deny` (licenses, advisories, duplicates) block merges; Dependabot on for Cargo, NuGet and Actions. | G11 | P0 |
| NFR-04 | CI strictness | `cargo fmt --check` and `cargo clippy -- -D warnings` must pass; `continue-on-error` removed. | G11 | P0 |
| NFR-05 | Measured performance | A reproducible bench script (ETW/WPR or `GetProcessTimes`) publishes per-release numbers. Targets: idle CPU < 0.5 % with 10 widgets, working set < 60 MB, cold start < 300 ms. Docs quote only measured numbers. | G10 | P1 |
| NFR-06 | Test coverage | Line coverage ≥ 70 % on `core_engine`, `ipc_protocol`, `system_providers`, `capability_broker` (via `cargo llvm-cov`); coverage report in CI. | — | P1 |
| NFR-07 | End-to-end tests | One Windows CI job starts the daemon, connects over the real pipe, loads a widget and checks a frame is presented. | — | P1 |
| NFR-08 | Reliability | Daemon survives Explorer restart, monitor hot-plug, GPU device-lost and sleep/resume with no manual action; 24 h soak test with < 5 MB memory growth. | — | P1 |
| NFR-09 | Accessibility | Dashboard meets WCAG 2.1 AA: `AutomationProperties.Name` on every control, full keyboard navigation, high-contrast theme; Accessibility Insights scan with 0 errors. | G12 | P1 |
| NFR-10 | Localization | All UI strings in `.resw`; English and one more language shipped; dates and numbers use system culture. | G12 | P2 |
| NFR-11 | Packaging and signing | Signed MSIX (or MSI) built in `release.yml`; binaries code-signed; SBOM (CycloneDX) attached to each release. | G14 | P1 |
| NFR-12 | Documentation integrity | Test counts and feature status generated from `cargo test -- --list` and code, not typed by hand; `LICENSE`, `SECURITY.md`, `.gitattributes` added; a link checker runs in CI. | G14 | P2 |

## Delivery plan to v0.8

Four milestones, in order. Security and CI come first because every later change should land behind strict checks. Each milestone ends with a gate that must pass before the next starts.

1. **M1 — Lock it down (security + CI).** FR-01, FR-02, FR-05, NFR-03, NFR-04.
   - Gate: a test client running as another process without the session token is refused; a Lua script calling `os.execute` fails to load; clippy and audit are green.
2. **M2 — Make the data real.** FR-03, FR-06, FR-07, FR-08, FR-09, NFR-01.
   - Gate: the synthetic-data grep passes; per-core CPU matches Task Manager; a captured sync blob is unreadable without the DPAPI key.
3. **M3 — Prove it.** FR-04, NFR-02, NFR-05, NFR-06, NFR-07, NFR-08.
   - Gate: a real WASM widget renders; benchmark script output replaces the hand-written numbers in README; 24 h soak passes.
4. **M4 — Polish and ship.** FR-10 to FR-14, NFR-09 to NFR-12.
   - Gate: signed installer installs and uninstalls cleanly on a fresh Windows 11 VM; accessibility scan has 0 errors; docs regenerated.

Rule for all milestones: a feature is "Completed" in `Project_Status.md` only when it has a test that runs against the real Windows API, not a fixture.

## Open questions

- [ ] Keep the 15 enterprise/AI/marketplace crates in scope, or freeze them until M1–M3 are done to shrink the surface?
- [ ] FR-10: ship a real ONNX model (adds roughly 10–50 MB) or rename the feature as rule-based?
- [ ] FR-09: which weather and market providers are acceptable (keyless vs. API key, terms of use)?
- [ ] NFR-05: are the performance targets above acceptable, or should they stay closer to the current README claims once measured?
- [ ] Should benchmark and test counts be removed from README until NFR-12 automation exists?

Scope note: this review read the code and docs but did not build or run the tests, because the connected machine has no Rust or .NET toolchain. Running `cargo test --workspace` on Windows is the first check worth doing.
