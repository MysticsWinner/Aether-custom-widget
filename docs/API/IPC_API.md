# Aether IPC API Reference

**JSON Schemas for `ControlCommand` Messages and Win32 Named Pipe Dispatches**

---

## 1. Named Pipe URI & Transport

- **Address**: `\\.\pipe\CustomWidgetEngineControlPipe`
- **Transport**: Win32 Named Pipe (Duplex Byte stream)
- **Serialization**: UTF-8 JSON lines (Rust Serde default externally tagged enum representation)

---

## 2. Serialization Format

Rust Serde's default serialization for `enum ControlCommand` uses **externally tagged representation**:
- **Unit variants** are encoded as simple JSON strings: `"Ping"`, `"GetStatus"`, `"ReloadAll"`, etc.
- **Variants with fields** are encoded as single-key JSON objects: `{"LoadWidget": {"manifest_path": "..."}}`.

---

## 3. Command Examples by Category

### Engine Diagnostics & Telemetry
```json
"Ping"
"GetStatus"
"GetDiagnostics"
"GetSubsystemHealth"
"GetHealthReport"
"GetWatchdogStatus"
```

### Desktop Widget Lifecycle & Display Options
```json
"ToggleDesktopWidget"
"ReloadAll"
"ListWidgets"
{"LoadWidget": {"manifest_path": "crates/perf_monitor_widget/widget.toml"}}
{"UnloadWidget": {"widget_id": "perf_monitor_widget"}}
{"DiscoverWidgets": {"search_paths": null}}
{"UpdateWidgetDisplayOptions": {"widget_id": "perf_monitor_widget", "opacity": 0.9, "scale": 1.0, "locked": false, "enabled": true}}
{"QuickSwapWidget": {"from_id": "widget_a", "to_id": "widget_b", "mode": "position"}}
{"EnableWidget": {"widget_id": "perf_monitor_widget"}}
{"DisableWidget": {"widget_id": "perf_monitor_widget"}}
{"SetWidgetOpacity": {"widget_id": "perf_monitor_widget", "opacity": 0.85}}
{"ResetWidgetConfig": {"widget_id": "perf_monitor_widget"}}
{"SetWidgetPosition": {"widget_id": "perf_monitor_widget", "x": 100, "y": 150}}
{"SetWidgetLock": {"widget_id": "perf_monitor_widget", "locked": true}}
{"ToggleWidgetLock": {"widget_id": "perf_monitor_widget"}}
{"HotReloadWidget": {"widget_id": "perf_monitor_widget"}}
{"InspectWidget": {"widget_id": "perf_monitor_widget"}}
{"GetExtendedWidgetInspector": {"widget_id": "perf_monitor_widget"}}
{"SetWidgetRenderConfig": {"widget_id": "perf_monitor_widget", "config_json": "{...}"}}
{"GetWidgetRenderConfig": {"widget_id": "perf_monitor_widget"}}
```

### Recovery, Safe Mode & Quarantine
```json
{"GetCrashHistory": {"widget_id": null}}
{"RollbackWidget": {"widget_id": "perf_monitor_widget"}}
"GetLaunchMode"
"ExitSafeMode"
"GetQuarantineList"
{"ReleaseQuarantine": {"widget_id": "perf_monitor_widget"}}
```

### Snapshots & Configuration
```json
{"CreateSnapshot": {"name": "Default Workstation"}}
"ListSnapshots"
{"RestoreSnapshot": {"snapshot_id": "snapshot_123"}}
{"DeleteSnapshot": {"snapshot_id": "snapshot_123"}}
{"ExportSnapshot": {"snapshot_id": "snapshot_123", "path": "C:\\backups\\snapshot.json"}}
{"ImportSnapshot": {"path": "C:\\backups\\snapshot.json"}}
```

### Security & Enterprise Policy
```json
{"RequestCapabilityToken": {"widget_id": "network_monitor", "capability": "NetworkAccess"}}
{"RevokeCapabilityToken": {"token_id": "tok_123"}}
{"GetWidgetResourceUsage": {"widget_id": "network_monitor"}}
{"GetCapabilityGrants": {"widget_id": null}}
"GetEnterprisePolicy"
{"UpdateEnterprisePolicy": {"policy_json": "{...}"}}
"GetAuditLogs"
"VerifyAuditChain"
```

### Event Recording & Observability
```json
"StartRecording"
"StopRecording"
"GetRecording"
{"ReplayRecording": {"from_seq": 0}}
{"InjectChaosFailure": {"scenario": "GpuDeviceLost"}}
"GetObservabilityStatus"
"GetPrometheusMetrics"
{"GenerateMinidump": {"reason": "ManualDiagnostics"}}
"ListMinidumps"
"GetSchedulerStatus"
{"SetAdaptiveTickMode": {"enabled": true}}
"GetResourceCacheStats"
{"ToggleLayoutGrid": {"enabled": true}}
```

### Themes, Profiles, Marketplace & AI
```json
{"SetThemeMode": {"mode": "dark"}}
{"ResolveDesignTokens": {"theme_id": null}}
{"SetAccessibilityMode": {"high_contrast": null, "reduce_motion": null, "reduce_transparency": null, "large_text": null}}
{"SetDesktopProfile": {"profile_id": "Coding"}}
"GetActiveProfile"
"ListProfiles"
{"SearchMarketplace": {"query": "monitoring", "category": null}}
{"SynthesizeDesktop": {"prompt": "Ultra minimal 4K trading workstation"}}
{"SynthesizeWidget": {"prompt": "Stock market ticker with sparklines"}}
{"GenerateWallpaperTheme": {"wallpaper_path": null}}
{"GetAiPerformanceAdvice": {"widget_id": null}}
```
