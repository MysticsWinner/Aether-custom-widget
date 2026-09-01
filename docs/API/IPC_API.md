# Aether IPC API Reference

**JSON Schemas for ControlCommand Messages and Named Pipe Dispatches**

---

## 1. Named Pipe URI & Transport

- **Address**: `\\.\pipe\CustomWidgetEngineControlPipe`
- **Protocol**: JSON UTF-8 lines (`\n`)

---

## 2. Command Reference

```json
// Ping / Pong
{"type": "Ping"}
{"type": "Pong"}

// Get Status & Telemetry
{"type": "GetStatus"}

// Widget Lifecycle
{"type": "LoadWidget", "payload": {"manifest_path": "crates/perf_monitor_widget/widget.toml"}}
{"type": "UnloadWidget", "payload": {"widget_id": "perf_monitor_widget"}}
{"type": "ReloadAll"}

// Theme Management
{"type": "SetThemeMode", "payload": {"mode": "Dark"}}
{"type": "ApplyThemeTokens", "payload": {"theme_id": "theme.cyberpunk.neon"}}

// Profiles
{"type": "SwitchProfile", "payload": {"profile_name": "Gaming"}}
{"type": "RollbackProfile"}

// AI Synthesis
{"type": "SynthesizeAiLayout", "payload": {"prompt": "Triple 4K Dev"}}
{"type": "SynthesizeAiTheme", "payload": {"prompt": "Cyberpunk Neon"}}
```
